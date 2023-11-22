use std::{
    cell::UnsafeCell,
    collections::BTreeMap,
    iter::{repeat, repeat_with},
    mem,
    ops::Range,
};

use recycle_vec::VecExt;
use resource_manager::ResourceId;
use rhai::Engine;
use self_cell::self_cell;
use smallvec::SmallVec;
use sound_engine::SoundConfig;

use crate::{
    connection::Primitive,
    errors::{NodeError, NodeWarning},
    global_state::{Resource, ResourceTypeAndIndex, Resources},
    midi_store::MidiStore,
    node::{
        Ins, MidiStoreInterface, MidisIndex, NodeIndex, NodeProcessContext, NodeRuntime, NodeState, Outs,
        StateInterface,
    },
    nodes::NodeVariant,
    traversal::calculate_traversal_order::NodeMappedIo,
};

use super::calculate_traversal_order::{Indexes, IoSpec};

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
    midi_io: Vec<UnsafeCell<Option<MidisIndex>>>,
    stream_default: ChunkedBuffer,
    midi_default: Vec<UnsafeCell<Option<MidisIndex>>>,
    value_default: Vec<UnsafeCell<Primitive>>,
}

#[derive(Debug)]
struct TraverserRefs<'io> {
    stream_sockets: Vec<&'io [&'io [UnsafeCell<f32>]]>,
    value_sockets: Vec<&'io [UnsafeCell<Primitive>]>,
    midi_sockets: Vec<&'io [UnsafeCell<Option<MidisIndex>>]>,
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
    let midi_io = repeat_with(|| UnsafeCell::new(None)).take(indexes.midi_count).collect();

    let stream_default = ChunkedBuffer::new(
        repeat_with(|| UnsafeCell::new(0.0)).take(config.buffer_size).collect(),
        |buffer| BufferChunks(repeat(&buffer[..]).take(indexes.max_stream_channels).collect()),
    );
    let midi_default = repeat_with(|| UnsafeCell::new(None))
        .take(indexes.max_midi_channels)
        .collect();
    let value_default = repeat_with(|| UnsafeCell::new(Primitive::None))
        .take(indexes.max_value_channels)
        .collect();

    TraverserIo {
        stream_io,
        value_io,
        midi_io,
        stream_default,
        midi_default,
        value_default,
    }
}

fn build_refs<'io>(io: &'io TraverserIo, indexes: &Indexes) -> TraverserRefs<'io> {
    let mut stream_sockets: Vec<&[&[UnsafeCell<f32>]]> = vec![];
    let mut midi_sockets: Vec<&[UnsafeCell<Option<MidisIndex>>]> = vec![];
    let mut value_sockets: Vec<&[UnsafeCell<Primitive>]> = vec![];

    for stream_config in &indexes.streams {
        if let Some(stream_config) = stream_config {
            stream_sockets.push(&io.stream_io.chunks()[stream_config.clone()])
        } else {
            stream_sockets.push(&io.stream_default.chunks()[..])
        }
    }

    for midi_config in &indexes.midis {
        if let Some(midi_config) = midi_config {
            midi_sockets.push(&io.midi_io[midi_config.clone()])
        } else {
            midi_sockets.push(&io.midi_default[..])
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
        midi_sockets,
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
    pub midi_in: Range<usize>,
    pub value_in: Range<usize>,
    pub stream_out: Range<usize>,
    pub midi_out: Range<usize>,
    pub value_out: Range<usize>,
    pub resources: Range<usize>,
    pub node: NodeVariant,
    pub values_to_input: SmallVec<[(usize, Primitive); 4]>,
}

#[derive(Debug)]
pub struct BufferedTraverser {
    nodes: Vec<TraverserNode>,
    node_to_index_mapping: BTreeMap<NodeIndex, usize>,
    resource_tracking: Vec<(ResourceId, Option<ResourceTypeAndIndex>)>,
    io_and_refs: IoAndRefs,
    store: MidiStore,
    config: SoundConfig,
    engine: Engine,
    time: i64,
    resource_scratch: Vec<Resource<'static>>,
    value_input_scratch: Vec<UnsafeCell<Primitive>>,
    value_ref_scratch: Vec<&'static [UnsafeCell<Primitive>]>,
}

impl BufferedTraverser {
    pub fn new<'a>(config: SoundConfig, mut io_spec: IoSpec, indexes: &Indexes, start_time: i64) -> BufferedTraverser {
        let mut node_to_index_mapping = BTreeMap::new();

        for (i, node_index) in io_spec.traversal_order.iter().enumerate() {
            node_to_index_mapping.insert(*node_index, i);
        }

        let io = build_io(config.clone(), indexes);
        let io_and_refs = IoAndRefs::new(io, |io| build_refs(io, indexes));

        let mut nodes: Vec<TraverserNode> = vec![];

        for index in io_spec.traversal_order.iter() {
            let spec = io_spec.nodes.remove(index).unwrap();
            let indexes = indexes.node_io[index].clone();

            nodes.push(TraverserNode {
                stream_in: indexes.stream_in,
                midi_in: indexes.midi_in,
                value_in: indexes.value_in,
                stream_out: indexes.stream_out,
                midi_out: indexes.midi_out,
                value_out: indexes.value_out,
                resources: indexes.resources,
                node: spec.node,
                values_to_input: SmallVec::new(),
            });
        }

        // TODO: the midi store params should be adjustable
        let store = MidiStore::new(50_000_000, 0);

        let engine = rhai::Engine::new();

        BufferedTraverser {
            nodes,
            node_to_index_mapping,
            resource_tracking: io_spec.resources_tracking,
            io_and_refs,
            store,
            config,
            engine,
            time: start_time,
            // TODO: initialize scratch with proper capacity
            resource_scratch: vec![],
            value_input_scratch: vec![],
            value_ref_scratch: vec![],
        }
    }

    pub fn step(
        &mut self,
        resources: &Resources,
        updated_node_states: Vec<(NodeIndex, serde_json::Value)>,
        graph_state: Option<&BTreeMap<NodeIndex, NodeState>>,
    ) {
        let mut errors: Vec<(NodeIndex, NodeError)> = vec![];
        let mut warnings: Vec<(NodeIndex, NodeWarning)> = vec![];

        let mut state_changes: Vec<(NodeIndex, NodeState)> = vec![];

        let TraverserRefs {
            stream_sockets,
            value_sockets,
            midi_sockets,
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

        for node in self.nodes.iter_mut() {
            let mut value_ref_scratch: Vec<&[UnsafeCell<Primitive>]> =
                mem::replace(&mut self.value_ref_scratch, vec![]).recycle();
            let mut value_input_scratch: Vec<UnsafeCell<Primitive>> =
                mem::replace(&mut self.value_input_scratch, vec![]).recycle();

            let value_inputs = if node.values_to_input.is_empty() {
                &value_sockets[node.value_in.clone()]
            } else {
                value_ref_scratch.extend(&value_sockets[node.value_in.clone()]);

                for (input_at, value) in node.values_to_input.drain(..) {
                    let new_idx = self.value_input_scratch.len();
                    value_input_scratch.push(UnsafeCell::new(value));

                    value_ref_scratch[input_at] = &self.value_input_scratch[new_idx..(new_idx + 1)];
                }

                &value_ref_scratch
            };

            node.node
                .process(
                    NodeProcessContext {
                        current_time: self.time,
                        resources,
                        script_engine: &self.engine,
                        external_state: StateInterface {
                            request_node_states: &mut || {},
                            enqueue_state_updates: &mut |_| {},
                            states: None,
                        },
                    },
                    unsafe {
                        Ins::new(
                            &midi_sockets[node.midi_in.clone()],
                            value_inputs,
                            &stream_sockets[node.stream_in.clone()],
                        )
                    },
                    unsafe {
                        Outs::new(
                            &midi_sockets[node.midi_out.clone()],
                            &value_sockets[node.value_out.clone()],
                            &stream_sockets[node.stream_out.clone()],
                        )
                    },
                    &mut MidiStoreInterface::new(&mut self.store),
                    &all_resources[node.resources.clone()],
                )
                .unwrap();

            self.value_ref_scratch = value_ref_scratch.recycle();
            self.value_input_scratch = value_input_scratch.recycle();
        }

        let debugging = self.io_and_refs.borrow_owner().stream_io.borrow_owner();
        println!("{:?}", unsafe {
            &*mem::transmute::<&[UnsafeCell<f32>], &UnsafeCell<[f32]>>(&debugging[..]).get()
        });

        self.time += self.config.buffer_size as i64;

        self.resource_scratch = all_resources.recycle();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use sound_engine::SoundConfig;

    use crate::{
        connection::{Socket, SocketType},
        global_state::Resources,
        graph_manager::GraphManager,
        traversal::calculate_traversal_order::{calc_indexes, calc_io_spec},
    };

    use super::BufferedTraverser;

    #[test]
    fn test_layout() {
        let mut manager = GraphManager::new();
        let (graph_index, _) = manager.new_graph().unwrap();
        let graph = manager.get_graph_mut(graph_index).unwrap();

        let (gain, _) = graph.add_node("GainNode").unwrap().value;
        let (midi, _) = graph.add_node("MidiToValuesNode").unwrap().value;
        let (osc, _) = graph.add_node("OscillatorNode").unwrap().value;

        graph
            .connect(
                midi,
                &Socket::Simple("frequency".into(), SocketType::Value, 1),
                osc,
                &Socket::Simple("frequency".into(), SocketType::Value, 1),
            )
            .unwrap();

        graph
            .connect(
                osc,
                &Socket::Simple("audio".into(), SocketType::Stream, 1),
                gain,
                &Socket::Simple("audio".into(), SocketType::Stream, 1),
            )
            .unwrap();

        let sound_config = SoundConfig {
            sample_rate: 48_000,
            buffer_size: 4,
        };

        let io_spec = calc_io_spec(
            manager.get_graph(graph_index).unwrap(),
            BTreeMap::new(),
            &sound_config,
            &mut rhai::Engine::new(),
            &Resources::default(),
            0,
            &manager,
            1,
        )
        .unwrap();
        let indexes = calc_indexes(&io_spec, graph_index, &manager).unwrap();

        let mut traverser = BufferedTraverser::new(sound_config.clone(), io_spec, &indexes, 0);
        traverser.step(&Resources::default(), vec![], None);
    }
}
