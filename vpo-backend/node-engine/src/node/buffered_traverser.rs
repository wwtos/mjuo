use std::{
    cell::UnsafeCell,
    collections::BTreeMap,
    fmt::Debug,
    iter::{repeat, repeat_with},
    mem,
    ops::Range,
    time::Duration,
};

use common::resource_manager::ResourceId;
use recycle_vec::VecExt;
use rhai::Engine;
use self_cell::self_cell;
use smallvec::SmallVec;
use sound_engine::SoundConfig;

use crate::{
    connection::{Primitive, Socket},
    errors::{ErrorsAndWarnings, NodeError},
    graph_manager::{GraphIndex, GraphManager},
    node::{Ins, NodeIndex, NodeProcessContext, NodeRuntime, NodeState, OscIndex, Outs, StateInterface},
    nodes::NodeVariant,
    resources::{Resource, ResourceTypeAndIndex, Resources},
};

use super::{
    calculate_traversal_order::{calc_indexes, generate_io_spec, Indexes},
    osc_store::OscStore,
};

#[derive(Debug)]
struct BufferChunks<'a>(Vec<&'a [UnsafeCell<f32>]>);
self_cell!(
    struct ChunkedBuffer {
        owner: Vec<UnsafeCell<f32>>,

        #[covariant]
        dependent: BufferChunks,
    }

    impl {Debug}
);

impl ChunkedBuffer {
    fn chunks(&self) -> &[&[UnsafeCell<f32>]] {
        &self.borrow_dependent().0[..]
    }
}

fn build_chunked_buffer(buffer: Vec<UnsafeCell<f32>>, chunk_size: usize) -> ChunkedBuffer {
    ChunkedBuffer::new(buffer, |buffer| BufferChunks(buffer.chunks_exact(chunk_size).collect()))
}

#[derive(Debug)]
struct TraverserIo {
    stream_io: ChunkedBuffer,
    value_io: Vec<UnsafeCell<Primitive>>,
    osc_io: Vec<UnsafeCell<Option<OscIndex>>>,
    stream_default: ChunkedBuffer,
    osc_default: Vec<UnsafeCell<Option<OscIndex>>>,
    value_default: Vec<UnsafeCell<Primitive>>,
}

#[derive(Debug)]
struct TraverserRefs<'io> {
    stream_sockets: Vec<&'io [&'io [UnsafeCell<f32>]]>,
    value_sockets: Vec<&'io [UnsafeCell<Primitive>]>,
    osc_sockets: Vec<&'io [UnsafeCell<Option<OscIndex>>]>,
}

fn build_io(config: SoundConfig, indexes: &Indexes) -> TraverserIo {
    let stream_io = build_chunked_buffer(
        repeat_with(|| UnsafeCell::new(0.0))
            .take(config.buffer_size * indexes.stream_count)
            .collect(),
        config.buffer_size,
    );
    let value_io = repeat_with(|| UnsafeCell::new(Primitive::None))
        .take(indexes.value_count)
        .collect();
    let osc_io = repeat_with(|| UnsafeCell::new(None)).take(indexes.osc_count).collect();

    let stream_default = ChunkedBuffer::new(
        repeat_with(|| UnsafeCell::new(0.0)).take(config.buffer_size).collect(),
        |buffer| BufferChunks(repeat(&buffer[..]).take(indexes.max_stream_channels).collect()),
    );
    let osc_default = repeat_with(|| UnsafeCell::new(None))
        .take(indexes.max_osc_channels)
        .collect();
    let value_default = repeat_with(|| UnsafeCell::new(Primitive::None))
        .take(indexes.max_value_channels)
        .collect();

    TraverserIo {
        stream_io,
        value_io,
        osc_io,
        stream_default,
        osc_default,
        value_default,
    }
}

fn build_refs<'io>(io: &'io TraverserIo, indexes: &Indexes) -> TraverserRefs<'io> {
    let mut stream_sockets: Vec<&[&[UnsafeCell<f32>]]> = vec![];
    let mut osc_sockets: Vec<&[UnsafeCell<Option<OscIndex>>]> = vec![];
    let mut value_sockets: Vec<&[UnsafeCell<Primitive>]> = vec![];

    for stream_config in &indexes.streams {
        if let Some(stream_config) = stream_config {
            stream_sockets.push(&io.stream_io.chunks()[stream_config.clone()])
        } else {
            stream_sockets.push(&io.stream_default.chunks()[..])
        }
    }

    for osc_config in &indexes.oscs {
        if let Some(osc_config) = osc_config {
            osc_sockets.push(&io.osc_io[osc_config.clone()])
        } else {
            osc_sockets.push(&io.osc_default[..])
        }
    }

    for value_config in &indexes.values {
        if let Some(value_config) = value_config {
            value_sockets.push(&io.value_io[value_config.clone()])
        } else {
            value_sockets.push(&io.value_default[..])
        }
    }

    TraverserRefs {
        stream_sockets,
        osc_sockets,
        value_sockets,
    }
}

self_cell!(
    struct IoAndRefs {
        owner: TraverserIo,

        #[covariant]
        dependent: TraverserRefs,
    }

    impl { Debug }
);

#[derive(Debug)]
struct TraverserNode {
    pub stream_in: Range<usize>,
    pub osc_in: Range<usize>,
    pub value_in: Range<usize>,
    pub stream_out: Range<usize>,
    pub osc_out: Range<usize>,
    pub value_out: Range<usize>,
    pub resources: Range<usize>,
    pub node: NodeVariant,
    pub values_to_input: SmallVec<[(usize, Primitive); 1]>,
    pub socket_lookup: BTreeMap<Socket, usize>,
}

pub struct StepResult {
    pub state_changes: Vec<(NodeIndex, NodeState)>,
    pub requested_state_updates: Vec<(NodeIndex, serde_json::Value)>,
    pub request_for_graph_state: bool,
}

pub struct BufferedTraverser {
    nodes: Vec<TraverserNode>,
    nodes_with_state: Vec<(usize, NodeIndex)>,
    node_to_index_mapping: BTreeMap<NodeIndex, usize>,
    resource_tracking: Vec<(ResourceId, Option<ResourceTypeAndIndex>)>,
    io_and_refs: IoAndRefs,
    osc_tracking: Vec<Option<OscIndex>>,
    config: SoundConfig,
    engine: Engine,
    time: Duration,
    resource_scratch: Vec<Resource<'static>>,
    value_input_scratch: Vec<UnsafeCell<Primitive>>,
    value_ref_scratch: Vec<&'static [UnsafeCell<Primitive>]>,
}

impl Debug for BufferedTraverser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferedTraverser").finish_non_exhaustive()
    }
}

unsafe impl Send for BufferedTraverser {}

impl BufferedTraverser {
    pub fn new<'a>(
        config: SoundConfig,
        manager: &GraphManager,
        graph_index: GraphIndex,
        resources: &Resources,
        start_time: Duration,
    ) -> Result<(ErrorsAndWarnings, BufferedTraverser), NodeError> {
        let graph = manager.get_graph(graph_index).unwrap();

        let (errors_and_warnings, mut io_spec) = generate_io_spec(
            graph,
            BTreeMap::new(),
            &config,
            &mut rhai::Engine::new(),
            resources,
            start_time,
            &manager,
            1,
        )?;
        let indexes = calc_indexes(&io_spec, graph)?;

        let mut node_to_index_mapping = BTreeMap::new();

        for (i, node_index) in io_spec.traversal_order.iter().enumerate() {
            node_to_index_mapping.insert(*node_index, i);
        }

        let io = build_io(config.clone(), &indexes);
        let io_and_refs = IoAndRefs::new(io, |io| build_refs(io, &indexes));

        let mut nodes: Vec<TraverserNode> = vec![];
        let mut nodes_with_state: Vec<(usize, NodeIndex)> = vec![];

        for (i, index) in io_spec.traversal_order.iter().enumerate() {
            let spec = io_spec.nodes.remove(index).unwrap();
            let indexes = indexes.node_io[index].clone();

            if spec.node.has_state() {
                nodes_with_state.push((i, *index));
            }

            nodes.push(TraverserNode {
                stream_in: indexes.stream_in,
                osc_in: indexes.osc_in,
                value_in: indexes.value_in,
                stream_out: indexes.stream_out,
                osc_out: indexes.osc_out,
                value_out: indexes.value_out,
                resources: indexes.resources,
                node: spec.node,
                values_to_input: spec.values_to_input,
                socket_lookup: spec.socket_lookup,
            });
        }

        let osc_io = &io_and_refs.borrow_owner().osc_io;
        let mut osc_tracking = Vec::with_capacity(osc_io.len());

        for index in osc_io {
            let index = index.get();
            osc_tracking.push(unsafe { (*index).as_ref().map(|x| x.private_clone()) });
        }

        let engine = rhai::Engine::new();

        Ok((
            errors_and_warnings,
            BufferedTraverser {
                nodes,
                nodes_with_state,
                node_to_index_mapping,
                resource_tracking: io_spec.resources_tracking,
                io_and_refs,
                osc_tracking,
                config,
                engine,
                time: start_time,
                // TODO: initialize scratch with proper capacity
                resource_scratch: vec![],
                value_input_scratch: vec![],
                value_ref_scratch: vec![],
            },
        ))
    }

    pub fn step(
        &mut self,
        resources: &Resources,
        updated_node_states: Vec<(NodeIndex, serde_json::Value)>,
        graph_state: Option<&BTreeMap<NodeIndex, NodeState>>,
        osc_store: &mut OscStore,
    ) -> StepResult {
        let mut state_changes: Vec<(NodeIndex, NodeState)> = vec![];

        let TraverserRefs {
            stream_sockets,
            value_sockets,
            osc_sockets,
        } = &self.io_and_refs.borrow_dependent();

        // get all the resources
        let mut all_resources: Vec<Resource> = mem::replace(&mut self.resource_scratch, vec![]).recycle();

        for (resource_id, possible_index) in self.resource_tracking.iter_mut() {
            let possible_resource = possible_index
                .as_ref()
                .and_then(|type_and_index| resources.get_resource(type_and_index));

            // grab the resource
            all_resources.push(if let Some(resource) = possible_resource {
                resource
            } else {
                // check if the resource is at a new location
                if let Some(new_resource_index) = resources.get_resource_index(resource_id) {
                    *possible_index = Some(new_resource_index);

                    if let Some(new_resource) = resources.get_resource(&new_resource_index) {
                        new_resource
                    } else {
                        Resource::NotFound
                    }
                } else {
                    // still doesn't exist
                    Resource::NotFound
                }
            });
        }

        // input updated node states
        for (node_index, new_node_state) in updated_node_states.into_iter() {
            let node = &mut self.nodes[self.node_to_index_mapping[&node_index]].node;
            node.set_state(new_node_state);
        }

        let mut requesting_graph_state = false;
        let mut requested_state_updates = vec![];

        for node in self.nodes.iter_mut() {
            let mut value_ref_scratch: Vec<&[UnsafeCell<Primitive>]> =
                mem::replace(&mut self.value_ref_scratch, vec![]).recycle();
            let mut value_input_scratch: Vec<UnsafeCell<Primitive>> =
                mem::replace(&mut self.value_input_scratch, vec![]).recycle();

            let value_inputs = if node.values_to_input.is_empty() {
                &value_sockets[node.value_in.clone()]
            } else {
                // create a custom `value_inputs` to inject changed valuse
                value_ref_scratch.extend(&value_sockets[node.value_in.clone()]);

                for (_, value) in &node.values_to_input {
                    value_input_scratch.push(UnsafeCell::new(value.clone()));
                }

                for (i, (input_at, _)) in node.values_to_input.drain(..).enumerate() {
                    value_ref_scratch[input_at] = &value_input_scratch[i..(i + 1)];
                }

                &value_ref_scratch
            };

            node.node.process(
                NodeProcessContext {
                    current_time: self.time,
                    resources,
                    script_engine: &self.engine,
                    external_state: StateInterface {
                        states: graph_state,
                        request_node_states: &mut || requesting_graph_state = true,
                        enqueue_state_updates: &mut |updates| requested_state_updates.extend(updates.into_iter()),
                    },
                },
                unsafe {
                    Ins::new(
                        &osc_sockets[node.osc_in.clone()],
                        value_inputs,
                        &stream_sockets[node.stream_in.clone()],
                    )
                },
                unsafe {
                    Outs::new(
                        &osc_sockets[node.osc_out.clone()],
                        &value_sockets[node.value_out.clone()],
                        &stream_sockets[node.stream_out.clone()],
                    )
                },
                osc_store,
                &all_resources[node.resources.clone()],
            );

            self.value_ref_scratch = value_ref_scratch.recycle();
            self.value_input_scratch = value_input_scratch.recycle();
        }

        for (vec_index, node_index) in &self.nodes_with_state {
            if let Some(new_node_state) = self.nodes[*vec_index].node.get_state() {
                state_changes.push((*node_index, new_node_state));
            }
        }

        // # Osc garbage collection
        //
        // As each osc bundle is "owned" by only the node that outputted it,
        // if the node is no longer outputting it it's good to be collected.
        let osc_io = &self.io_and_refs.borrow_owner().osc_io;
        for (last_osc_index, new_osc_index) in self.osc_tracking.iter_mut().zip(osc_io.iter()) {
            // SAFETY: io_and_refs isn't being used by anything currently (since we're running in a
            // single thread)
            let new_osc_index = unsafe { &*new_osc_index.get() };

            if last_osc_index != new_osc_index {
                if let Some(some_index) = last_osc_index {
                    osc_store.remove_osc(some_index.private_clone());
                }

                *last_osc_index = new_osc_index.as_ref().map(|x| x.private_clone());
            }
        }

        // TODO: make sure this won't drift over time
        let advance_time = Duration::from_secs_f64(self.config.buffer_size as f64 / self.config.sample_rate as f64);
        self.time += advance_time;

        self.resource_scratch = all_resources.recycle();

        StepResult {
            state_changes,
            request_for_graph_state: requesting_graph_state,
            requested_state_updates: requested_state_updates,
        }
    }

    pub fn input_value_default(
        &mut self,
        node_index: NodeIndex,
        socket: &Socket,
        value: Primitive,
    ) -> Result<(), NodeError> {
        let mapped_index = self.node_to_index_mapping.get(&node_index);

        if let Some(mapped_index) = mapped_index {
            let value_index = self.nodes[*mapped_index].socket_lookup.get(socket);

            if let Some(value_index) = value_index.copied() {
                self.nodes[*mapped_index].values_to_input.push((value_index, value));

                Ok(())
            } else {
                Err(NodeError::SocketDoesNotExist { socket: socket.clone() })
            }
        } else {
            Err(NodeError::NodeDoesNotExist { node_index })
        }
    }

    pub fn get_node_mut(&mut self, index: NodeIndex) -> Option<&mut NodeVariant> {
        self.node_to_index_mapping
            .get(&index)
            .map(|internal_index| &mut self.nodes[*internal_index].node)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use sound_engine::SoundConfig;

    use crate::{
        connection::{Socket, SocketType},
        graph_manager::GraphManager,
        node::osc_store::OscStore,
        resources::Resources,
    };

    use super::BufferedTraverser;

    #[test]
    fn test_layout() {
        let mut manager = GraphManager::new(1);
        let (graph_index, _) = manager.new_graph().unwrap();
        let graph = manager.get_graph_mut(graph_index).unwrap();
        let mut osc_store = OscStore::new(256, 0);

        let (gain, _) = graph.add_node("GainNode").unwrap().value;
        let (osc, _) = graph.add_node("MidiToValuesNode").unwrap().value;
        let (oscil, _) = graph.add_node("OscillatorNode").unwrap().value;

        graph
            .connect(
                osc,
                &Socket::Simple("frequency".into(), SocketType::Value, 1),
                oscil,
                &Socket::Simple("frequency".into(), SocketType::Value, 1),
            )
            .unwrap();

        graph
            .connect(
                oscil,
                &Socket::Simple("audio".into(), SocketType::Stream, 1),
                gain,
                &Socket::Simple("audio".into(), SocketType::Stream, 1),
            )
            .unwrap();

        let sound_config = SoundConfig {
            sample_rate: 48_000,
            buffer_size: 4,
        };

        let (_, mut traverser) = BufferedTraverser::new(
            sound_config.clone(),
            &manager,
            graph_index,
            &Resources::default(),
            Duration::ZERO,
        )
        .unwrap();
        traverser.step(&Resources::default(), vec![], None, &mut osc_store);
        traverser.step(&Resources::default(), vec![], None, &mut osc_store);
    }
}
