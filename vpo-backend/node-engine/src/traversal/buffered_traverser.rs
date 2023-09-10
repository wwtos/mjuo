use core::slice;
use std::{
    any::Any,
    collections::BTreeMap,
    fmt::Debug,
    iter::repeat,
    mem::{self, MaybeUninit},
};

use resource_manager::{ResourceId, ResourceIndex};
use rhai::Engine;
use smallvec::SmallVec;
use sound_engine::SoundConfig;

use crate::{
    connection::{MidiBundle, Primitive, Socket, SocketType},
    errors::{ErrorsAndWarnings, NodeError, NodeOk, NodeWarning},
    global_state::{ResourceType, Resources},
    graph_manager::{GraphIndex, GraphManager},
    node::{Ins, NodeIndex, NodeInitParams, NodeProcessContext, NodeRow, NodeRuntime, NodeState, Outs, StateInterface},
    nodes::{new_variant, NodeVariant},
};

use super::calculate_traversal_order::calculate_graph_traverse_order;

const BUFFER_SIZE: usize = 128;

#[derive(Debug, Clone)]
struct AdvanceBy {
    pub inputs: usize,
    pub outputs: usize,
}

pub struct TraverserResult {
    pub errors_and_warnings: ErrorsAndWarnings,
    pub state_changes: Vec<(NodeIndex, NodeState)>,
    pub requested_state_updates: Vec<(NodeIndex, serde_json::Value)>,
    pub request_for_graph_state: bool,
}

#[derive(Debug, Clone)]
struct NodeAssociatedLocations {
    pub value_socket_to_index: Vec<(Socket, usize)>,
    pub stream_outputs_index: usize,
    pub stream_output_sockets: Vec<Socket>,
    pub midi_outputs_index: usize,
    pub midi_output_sockets: Vec<Socket>,
    pub value_outputs_index: usize,
    pub value_output_sockets: Vec<Socket>,
    pub vec_index: usize,
}

#[derive(Debug, Clone, Default)]
struct NodeTraversalWrapper {
    pub node: NodeVariant,
    /// A mapping of a value to input to its location
    pub values_to_input: SmallVec<[(usize, Primitive); 4]>,
}

#[derive(Clone, Default)]
pub struct BufferedTraverser {
    buffer_size: usize,

    nodes: Vec<NodeTraversalWrapper>,
    node_indexes: Vec<NodeIndex>,
    nodes_linked_to_ui: Vec<(usize, NodeIndex)>,

    node_to_location_mapping: BTreeMap<NodeIndex, NodeAssociatedLocations>,

    midi_outputs: Vec<Option<MidiBundle>>,
    value_outputs: Vec<Option<Primitive>>,
    stream_outputs: Vec<f32>,

    /// If None, it's not connected to anything (to keep alignment when inputting into node)
    stream_input_mappings: Vec<Option<usize>>,
    stream_advance_by: Vec<AdvanceBy>,
    /// If None, it's not connected to anything (to keep alignment when inputting into node)
    midi_input_mappings: Vec<Option<usize>>,
    midi_advance_by: Vec<AdvanceBy>,
    /// If None, it's not connected to anything (to keep alignment when inputting into node)
    value_input_mappings: Vec<Option<usize>>,
    value_advance_by: Vec<AdvanceBy>,

    resource_tracking: Vec<(ResourceId, Option<(ResourceType, ResourceIndex)>)>,
    resource_advance_by: Vec<usize>,
}

impl Debug for BufferedTraverser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BufferedTraverser {{ ... }}")
    }
}

impl BufferedTraverser {
    pub fn new(
        graph_index: GraphIndex,
        graph_manager: &GraphManager,
        script_engine: &Engine,
        resources: &Resources,
        current_time: i64,
        sound_config: SoundConfig,
    ) -> Result<(BufferedTraverser, ErrorsAndWarnings), NodeError> {
        let mut traverser = BufferedTraverser::default();

        traverser
            .init_graph(
                graph_index,
                graph_manager,
                script_engine,
                resources,
                current_time,
                sound_config,
            )
            .map(|errors_and_warnings| (traverser, errors_and_warnings))
    }

    // things are stored as follows:
    // outputs: [node1_outputs]|[node2_ouputs]...
    // input_mappings maps to the position of each of the node's inputs
    pub fn init_graph(
        &mut self,
        graph_index: GraphIndex,
        graph_manager: &GraphManager,
        script_engine: &Engine,
        resources: &Resources,
        current_time: i64,
        sound_config: SoundConfig,
    ) -> Result<ErrorsAndWarnings, NodeError> {
        self.buffer_size = sound_config.buffer_size;

        let graph = graph_manager.get_graph(graph_index)?;

        // figure out what order we should go through the nodes
        let traversal_order = calculate_graph_traverse_order(&graph);

        // pull out the old nodes (don't recreate them every time)
        let mut old_nodes: BTreeMap<NodeIndex, NodeTraversalWrapper> =
            self.node_indexes.drain(0..).zip(self.nodes.drain(0..)).collect();

        self.nodes.clear();
        self.node_indexes.clear();
        self.node_to_location_mapping.clear();
        self.stream_outputs.clear();
        self.stream_input_mappings.clear();
        self.stream_advance_by.clear();
        self.midi_outputs.clear();
        self.midi_input_mappings.clear();
        self.midi_advance_by.clear();
        self.value_outputs.clear();
        self.value_input_mappings.clear();
        self.value_advance_by.clear();
        self.nodes_linked_to_ui.clear();
        self.resource_tracking.clear();
        self.resource_advance_by.clear();

        let mut errors: Vec<(NodeIndex, NodeError)> = vec![];
        let mut warnings: Vec<(NodeIndex, NodeWarning)> = vec![];

        // now for the fun part ;)
        // # Step 1, denormalize all of the nodes
        // Each of the different types of input is split up across a different array, lined up
        // back to back
        for (vec_index, node_index) in traversal_order.iter().enumerate() {
            // create and init the node
            let node_instance = graph.get_node(*node_index)?;

            let mut variant = if let Some(previous_node) = old_nodes.remove(node_index) {
                previous_node.node
            } else {
                new_variant(&node_instance.get_node_type(), &sound_config)?
            };

            // get the child graph info, if any
            let child_graph_info = node_instance.get_child_graph();

            let init_result_res = variant.init(NodeInitParams {
                props: node_instance.get_properties(),
                script_engine,
                resources,
                current_time,
                graph_manager,
                sound_config: &sound_config,
                node_state: node_instance.get_state(),
                child_graph: child_graph_info.clone(),
            });

            // handle any errors from initializing the node
            let needed_resources = match init_result_res {
                Ok(init_result) => {
                    for warning in init_result.warnings.into_iter() {
                        warnings.push((*node_index, warning))
                    }

                    init_result.value.needed_resources
                }
                Err(err) => {
                    errors.push((*node_index, err));

                    vec![]
                }
            };

            for needed_resource in &needed_resources {
                let resource_index = resources.get_resource_index(needed_resource);

                self.resource_tracking.push((needed_resource.clone(), resource_index));
            }

            if variant.has_state() {
                self.nodes_linked_to_ui.push((vec_index, *node_index));
            }

            let mut stream_inputs = 0;
            let mut midi_inputs = 0;
            let mut value_inputs = 0;

            let mut value_socket_to_index = vec![];

            let mut to_input: SmallVec<[(usize, Primitive); 4]> = SmallVec::new();

            // go through the node by all its inputs
            for socket in node_instance.list_input_sockets() {
                let default_row = node_instance.get_default(socket).unwrap();

                if let NodeRow::Input(socket, default) = default_row {
                    let is_connected = graph.get_input_connection_index(*node_index, &socket)?.is_some();

                    match socket.socket_type() {
                        SocketType::Stream => {
                            stream_inputs += 1;
                        }
                        SocketType::Midi => {
                            midi_inputs += 1;
                        }
                        SocketType::Value => {
                            if !is_connected {
                                to_input.push((value_inputs, default.as_value().unwrap()));
                            }

                            value_socket_to_index.push((socket, value_inputs));

                            value_inputs += 1;
                        }
                        _ => {}
                    }
                }
            }

            // create a list of all of the outputs
            let mut stream_output_sockets = vec![];
            let mut midi_output_sockets = vec![];
            let mut value_output_sockets = vec![];

            for socket in node_instance.list_output_sockets() {
                match socket.socket_type() {
                    SocketType::Stream => stream_output_sockets.push(socket.clone()),
                    SocketType::Midi => midi_output_sockets.push(socket.clone()),
                    SocketType::Value => value_output_sockets.push(socket.clone()),
                    _ => {}
                }
            }

            self.nodes.push(NodeTraversalWrapper {
                node: variant,
                values_to_input: to_input,
            });
            self.node_indexes.push(*node_index);

            self.node_to_location_mapping.insert(
                *node_index,
                NodeAssociatedLocations {
                    stream_outputs_index: self.stream_outputs.len(),
                    stream_output_sockets: stream_output_sockets.clone(),
                    midi_outputs_index: self.midi_outputs.len(),
                    midi_output_sockets: midi_output_sockets.clone(),
                    value_outputs_index: self.value_outputs.len(),
                    value_output_sockets: value_output_sockets.clone(),
                    value_socket_to_index,
                    vec_index,
                },
            );

            // figure out how much the traverser needs to advance between each node
            self.stream_advance_by.push(AdvanceBy {
                inputs: stream_inputs,
                outputs: stream_output_sockets.len(),
            });

            self.midi_advance_by.push(AdvanceBy {
                inputs: midi_inputs,
                outputs: midi_output_sockets.len(),
            });

            self.value_advance_by.push(AdvanceBy {
                inputs: value_inputs,
                outputs: value_output_sockets.len(),
            });

            self.resource_advance_by.push(needed_resources.len());

            // fill the outputs with defaults
            self.stream_outputs
                .extend(repeat(0.0).take(stream_output_sockets.len() * self.buffer_size));
            self.midi_outputs.extend(repeat(None).take(midi_output_sockets.len()));
            self.value_outputs.extend(repeat(None).take(value_output_sockets.len()));
        }

        // # Step 2, populate mappings between nodes
        // Now we know where all the nodes are, so we can tell the each node where it can
        // find its input from
        for index in traversal_order.iter() {
            let wrapper = graph.get_node(*index)?;

            // let's look through this node's inputs
            for input in wrapper.list_input_sockets() {
                // is this node's input socket connected to anything?
                if let Some(connection_index) = graph.get_input_connection_index(*index, input)? {
                    // get the node that it's connected from
                    let connection = graph.get_graph().get_edge(connection_index.0).expect("edge to exist");
                    let from = NodeIndex(connection.get_from());

                    // where is the other nodes' output location?
                    let other_outputs = self.node_to_location_mapping.get(&from).unwrap();

                    // add it to the mapping
                    match input.socket_type() {
                        SocketType::Stream => {
                            let position_in_stream = other_outputs
                                .stream_output_sockets
                                .iter()
                                .position(|other_socket| other_socket == &connection.data.from_socket)
                                .unwrap()
                                * self.buffer_size
                                + other_outputs.stream_outputs_index;

                            self.stream_input_mappings.push(Some(position_in_stream));
                        }
                        SocketType::Midi => {
                            let position_in_midi = other_outputs
                                .midi_output_sockets
                                .iter()
                                .position(|other_socket| other_socket == &connection.data.from_socket)
                                .unwrap()
                                + other_outputs.midi_outputs_index;

                            self.midi_input_mappings.push(Some(position_in_midi));
                        }
                        SocketType::Value => {
                            let position_in_value = other_outputs
                                .value_output_sockets
                                .iter()
                                .position(|other_socket| other_socket == &connection.data.from_socket)
                                .unwrap()
                                + other_outputs.value_outputs_index;

                            self.value_input_mappings.push(Some(position_in_value));
                        }
                        SocketType::NodeRef => {}
                    }
                } else {
                    // it's not connected to anything, so push None in the mapping
                    // (to preserve alignment when things are inputted into the node)
                    match input.socket_type() {
                        SocketType::Stream => {
                            self.stream_input_mappings.push(None);
                        }
                        SocketType::Midi => {
                            self.midi_input_mappings.push(None);
                        }
                        SocketType::Value => {
                            self.value_input_mappings.push(None);
                        }
                        SocketType::NodeRef => {}
                    }
                }
            }
        }

        if !errors.is_empty() {
            println!("errors: {:#?}", errors);
        }

        Ok(ErrorsAndWarnings { errors, warnings })
    }

    pub fn traverse(
        &mut self,
        current_time: i64,
        script_engine: &Engine,
        resources: &Resources,
        updated_node_states: Vec<(NodeIndex, serde_json::Value)>,
        graph_state: Option<&BTreeMap<NodeIndex, NodeState>>,
    ) -> TraverserResult {
        let mut errors: Vec<(NodeIndex, NodeError)> = vec![];
        let mut warnings: Vec<(NodeIndex, NodeWarning)> = vec![];

        let mut state_changes: Vec<(NodeIndex, NodeState)> = vec![];

        // used as a default pointer if a node doesn't have an input connected
        let nothing_stream = vec![0.0_f32; self.buffer_size];
        let nothing_midi = None;
        let nothing_value = None;

        let mut midi_mapping_i = 0;
        let mut value_mapping_i = 0;
        let mut stream_mapping_i = 0;

        let mut resource_input_i = 0;

        let mut midi_inputs: [&Option<MidiBundle>; BUFFER_SIZE] = [&nothing_midi; BUFFER_SIZE];
        let mut value_inputs: [&Option<Primitive>; BUFFER_SIZE] = [&nothing_value; BUFFER_SIZE];
        let mut value_staging: [Option<Primitive>; BUFFER_SIZE] = staging_values();
        let mut stream_inputs: [&[f32]; BUFFER_SIZE] = [&nothing_stream; BUFFER_SIZE];
        let mut resource_inputs: [MaybeUninit<Option<(ResourceIndex, &dyn Any)>>; BUFFER_SIZE] =
            unsafe { MaybeUninit::uninit().assume_init() };

        let mut midi_outputs_i = 0;
        let mut value_outputs_i = 0;
        let mut stream_outputs_i = 0;

        let mut stream_outputs: [MaybeUninit<&mut [f32]>; BUFFER_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };

        let mut requesting_graph_state = false;
        let mut requested_state_updates = vec![];

        // input updated node states
        for (node_index, new_node_state) in updated_node_states.into_iter() {
            let node = &mut self.nodes[self.node_to_location_mapping.get(&node_index).unwrap().vec_index];
            node.node.set_state(new_node_state);
        }

        for (i, node) in self.nodes.iter_mut().enumerate() {
            // input resources
            for j in 0..self.resource_advance_by[i] {
                let (resource_id, possible_index) = &self.resource_tracking[resource_input_i];

                let possible_resource = possible_index.as_ref().and_then(|(resource_type, resource_index)| {
                    resources
                        .get_any(resource_type, *resource_index)
                        .map(|resource| (resource_index, resource))
                });

                // does it exist at the index?
                let to_input = if let Some((index, resource)) = possible_resource {
                    Some((*index, resource))
                } else {
                    // else check to see if it has a new index
                    if let Some(new_resource_index) = resources.get_resource_index(resource_id) {
                        self.resource_tracking[resource_input_i].1 = Some(new_resource_index.clone());

                        resources
                            .get_any(&new_resource_index.0, new_resource_index.1)
                            .map(|resource| (new_resource_index.1, resource))
                    } else {
                        // still doesn't exist
                        None
                    }
                };

                resource_inputs[j].write(to_input);

                resource_input_i += 1;
            }

            let midi_input_count = self.midi_advance_by[i].inputs;
            let value_input_count = self.value_advance_by[i].inputs;
            let stream_input_count = self.stream_advance_by[i].inputs;

            let stream_output_count = self.stream_advance_by[i].outputs;
            let value_output_count = self.value_advance_by[i].outputs;
            let midi_output_count = self.midi_advance_by[i].outputs;

            let midi_output_index = midi_outputs_i;
            let value_output_index = value_outputs_i;

            // clear last outputs (up to what we'll be using)
            self.midi_outputs[midi_output_index..(midi_output_index + midi_output_count)].fill(None);
            self.value_outputs[value_output_index..(value_output_index + value_output_count)].fill(None);

            let midi_ptr = self.midi_outputs.as_mut_ptr();
            let value_ptr = self.value_outputs.as_mut_ptr();
            let value_staging_ptr = value_staging.as_mut_ptr();
            let streams_ptr = self.stream_outputs.as_mut_ptr();

            // set up midi and value inputs
            for j in 0..midi_input_count {
                if let Some(input_index) = self.midi_input_mappings[midi_mapping_i] {
                    // SAFETY: make sure we don't exceed the midi output's length
                    assert!(input_index < self.midi_outputs.len());

                    midi_inputs[j] = unsafe { &*midi_ptr.add(input_index) };
                } else {
                    midi_inputs[j] = &nothing_midi;
                }

                midi_mapping_i += 1;
            }

            for j in 0..value_input_count {
                if let Some(input_index) = self.value_input_mappings[value_mapping_i] {
                    value_inputs[j] = unsafe { &*value_ptr.add(input_index) };
                } else {
                    value_inputs[j] = &nothing_value;
                }

                value_mapping_i += 1;
            }

            // override any values coming in with values from the user, if any
            for (j, (input_at, override_input)) in node.values_to_input.drain(..).enumerate() {
                let staging_ref = unsafe { &mut *value_staging_ptr.add(j) };
                *staging_ref = Some(override_input);
                value_inputs[input_at] = staging_ref;
            }

            // build the list of input references from other nodes' outputs
            for j in 0..stream_input_count {
                if let Some(input_index) = self.stream_input_mappings[stream_mapping_i] {
                    // SAFETY: Make sure we don't have a slice exceed the length of the array
                    assert!(input_index + self.buffer_size <= self.stream_outputs.len());

                    stream_inputs[j] = unsafe { slice::from_raw_parts(streams_ptr.add(input_index), self.buffer_size) };
                } else {
                    stream_inputs[j] = &nothing_stream;
                }

                stream_mapping_i += 1;
            }

            // ...and the list of output references
            for j in 0..stream_output_count {
                let output_index = stream_outputs_i + j * self.buffer_size;

                // SAFETY: Make sure we don't have a slice exceed the length of the array
                assert!(output_index + self.buffer_size <= self.stream_outputs.len());

                stream_outputs[j]
                    .write(unsafe { slice::from_raw_parts_mut(streams_ptr.add(output_index), self.buffer_size) });
            }

            // FINALLY
            let res = node.node.process(
                NodeProcessContext {
                    current_time,
                    script_engine,
                    resources,
                    external_state: StateInterface {
                        states: graph_state,
                        request_node_states: &mut || requesting_graph_state = true,
                        enqueue_state_updates: &mut |updates| requested_state_updates.extend(updates.into_iter()),
                    },
                },
                // SAFETY: we've already initialized 0..inputs and 0..outputs above
                Ins {
                    midis: &midi_inputs[0..midi_input_count],
                    values: &value_inputs[0..value_input_count],
                    streams: &stream_inputs[0..stream_input_count],
                },
                Outs {
                    midis: unsafe {
                        slice::from_raw_parts_mut(
                            midi_ptr.add(midi_output_index),
                            midi_output_index + midi_output_count,
                        )
                    },
                    values: unsafe {
                        slice::from_raw_parts_mut(
                            value_ptr.add(value_output_index),
                            value_output_index + value_output_count,
                        )
                    },
                    streams: unsafe {
                        mem::transmute::<_, &mut [&mut [f32]]>(&mut stream_outputs[0..stream_output_count])
                    },
                },
                unsafe {
                    mem::transmute::<
                        &[MaybeUninit<Option<(ResourceIndex, &dyn Any)>>],
                        &[Option<(ResourceIndex, &dyn Any)>],
                    >(&resource_inputs[0..self.resource_advance_by[i]])
                },
            );

            match res {
                Ok(NodeOk {
                    warnings: mut node_warnings,
                    ..
                }) => {
                    for warning in node_warnings.drain(..) {
                        warnings.push((self.node_indexes[i], warning));
                    }
                }
                Err(err) => {
                    errors.push((self.node_indexes[i], err));
                }
            }

            midi_outputs_i += self.midi_advance_by[i].outputs;
            value_outputs_i += self.value_advance_by[i].outputs;
            stream_outputs_i += self.stream_advance_by[i].outputs * self.buffer_size;
        }

        for (vec_index, node_index) in &self.nodes_linked_to_ui {
            if let Some(new_node_state) = self.nodes[*vec_index].node.get_state() {
                state_changes.push((*node_index, new_node_state));
            }
        }

        TraverserResult {
            errors_and_warnings: ErrorsAndWarnings { errors, warnings },
            state_changes,
            request_for_graph_state: requesting_graph_state,
            requested_state_updates: requested_state_updates,
        }
    }

    pub fn get_node_mut(&mut self, index_to_find: NodeIndex) -> Option<&mut NodeVariant> {
        self.nodes
            .iter_mut()
            .zip(&self.node_indexes)
            .find(|(_, index)| *index == &index_to_find)
            .map(|(node, _)| &mut node.node)
    }

    pub fn input_value_default(
        &mut self,
        node_index: NodeIndex,
        socket: &Socket,
        value: Primitive,
    ) -> Result<(), NodeError> {
        let locations = self.node_to_location_mapping.get(&node_index);

        if let Some(locations) = locations {
            let value_index = locations
                .value_socket_to_index
                .iter()
                .find_map(|(possible_socket, index)| if possible_socket == socket { Some(*index) } else { None });

            if let Some(value_index) = value_index {
                self.nodes[locations.vec_index]
                    .values_to_input
                    .push((value_index, value));

                Ok(())
            } else {
                Err(NodeError::SocketDoesNotExist { socket: socket.clone() })
            }
        } else {
            Err(NodeError::NodeDoesNotExist { node_index })
        }
    }
}

fn staging_values() -> [Option<Primitive>; BUFFER_SIZE] {
    let mut uninited: [MaybeUninit<Option<Primitive>>; BUFFER_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };

    for value in uninited.iter_mut() {
        value.write(None);
    }

    unsafe {
        mem::transmute::<[MaybeUninit<Option<Primitive>>; BUFFER_SIZE], [Option<Primitive>; BUFFER_SIZE]>(uninited)
    }
}
