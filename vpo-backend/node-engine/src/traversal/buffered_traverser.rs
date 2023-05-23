use core::slice;
use std::{
    collections::BTreeMap,
    iter::repeat,
    mem::{self, MaybeUninit},
};

use rhai::Engine;
use smallvec::SmallVec;
use sound_engine::SoundConfig;

use crate::{
    connection::{MidiBundle, Primitive, Socket, SocketType},
    errors::{ErrorsAndWarnings, NodeError, NodeOk, NodeWarning},
    global_state::Resources,
    graph_manager::{GraphIndex, GraphManager},
    node::{NodeIndex, NodeInitState, NodeProcessState, NodeRow, NodeRuntime, NodeState, StateInterface},
    nodes::variants::{new_variant, NodeVariant},
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
struct Locations {
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
    pub to_input: SmallVec<[(usize, Primitive); 4]>,
}

#[derive(Debug, Clone, Default)]
pub struct BufferedTraverser {
    buffer_size: usize,

    nodes: Vec<NodeTraversalWrapper>,
    node_indexes: Vec<NodeIndex>,
    nodes_linked_to_ui: Vec<(usize, NodeIndex)>,

    node_to_location_mapping: BTreeMap<NodeIndex, Locations>,

    stream_outputs: Vec<f32>,
    midi_outputs: Vec<Option<MidiBundle>>,
    value_outputs: Vec<Option<Primitive>>,

    /// If none, it's not connected to anything
    stream_input_mappings: Vec<Option<usize>>,
    stream_advance_by: Vec<AdvanceBy>,
    midi_input_mappings: Vec<Option<usize>>,
    midi_advance_by: Vec<AdvanceBy>,
    value_input_mappings: Vec<Option<usize>>,
    value_advance_by: Vec<AdvanceBy>,
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
        let mut old_nodes = self.nodes.drain(0..).zip(self.node_indexes.drain(0..)).fold(
            BTreeMap::new(),
            |mut map, (node, node_index)| {
                map.insert(node_index, node);

                map
            },
        );

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

        let mut errors: Vec<(NodeIndex, NodeError)> = vec![];
        let mut warnings: Vec<(NodeIndex, NodeWarning)> = vec![];

        // now for the fun part
        for (vec_index, node_index) in traversal_order.iter().enumerate() {
            // create and init the node
            let node_wrapper = graph.get_node(*node_index)?;

            let mut variant = if let Some(previous_node) = old_nodes.remove(node_index) {
                previous_node.node
            } else {
                new_variant(&node_wrapper.get_node_type(), &sound_config)?
            };

            // get the child graph info, if any
            let child_graph_info = node_wrapper.get_child_graph_info();

            let init_result_res = variant.init(
                NodeInitState {
                    props: node_wrapper.get_properties(),
                    script_engine,
                    resources,
                    current_time,
                    graph_manager,
                    sound_config: &sound_config,
                    state: node_wrapper.get_state(),
                },
                child_graph_info,
            );

            // handle any errors from initializing the node
            match init_result_res {
                Ok(init_result) => {
                    for warning in init_result.warnings.into_iter() {
                        warnings.push((*node_index, warning))
                    }
                }
                Err(err) => {
                    errors.push((*node_index, err));
                }
            };

            if variant.has_state() {
                self.nodes_linked_to_ui.push((vec_index, *node_index));
            }

            let mut stream_inputs = 0;
            let mut midi_inputs = 0;
            let mut value_inputs = 0;

            let mut value_socket_to_index = vec![];

            let mut to_input: SmallVec<[(usize, Primitive); 4]> = SmallVec::new();

            // go through the node by all its inputs
            for socket in node_wrapper.list_input_sockets() {
                let default_row = node_wrapper.get_default(socket).unwrap();

                if let NodeRow::Input(socket, default) = default_row {
                    let is_connected = graph.get_input_connection_index(*node_index, socket)?.is_some();

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

            // create a list of its default inputs and count the outputs
            let mut stream_output_sockets = vec![];
            let mut midi_output_sockets = vec![];
            let mut value_output_sockets = vec![];

            for socket in node_wrapper.list_output_sockets() {
                match socket.socket_type() {
                    SocketType::Stream => stream_output_sockets.push(socket),
                    SocketType::Midi => midi_output_sockets.push(socket),
                    SocketType::Value => value_output_sockets.push(socket),
                    _ => {}
                }
            }

            self.nodes.push(NodeTraversalWrapper {
                node: variant,
                to_input,
            });
            self.node_indexes.push(*node_index);

            self.node_to_location_mapping.insert(
                *node_index,
                Locations {
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

            self.stream_outputs
                .extend(repeat(0.0).take(stream_output_sockets.len() * self.buffer_size));
            self.midi_outputs.extend(repeat(None).take(midi_output_sockets.len()));
            self.value_outputs.extend(repeat(None).take(value_output_sockets.len()));
        }

        // the next step is to populate the input mappings, since we know where all the nodes are now
        // The input mappings is a mapping to get the node's next input
        for index in traversal_order.iter() {
            let wrapper = graph.get_node(*index)?;

            // let's look through this node's inputs
            for input in wrapper.list_input_sockets() {
                // is this socket connected to anything?
                if let Some(connection_index) = graph.get_input_connection_index(*index, input)? {
                    // get what it's connected from
                    let connection = graph.get_graph().get_edge(connection_index.0)?;
                    let from = NodeIndex(connection.get_from());

                    // where is the other nodes' output location?
                    let other_outputs = self.node_to_location_mapping.get(&from).unwrap();

                    // add it to the mapping
                    match input.socket_type() {
                        SocketType::Stream => {
                            let position_in_stream = other_outputs
                                .stream_output_sockets
                                .iter()
                                .position(|&other_socket| other_socket == connection.data.from_socket)
                                .unwrap()
                                * self.buffer_size
                                + other_outputs.stream_outputs_index;

                            self.stream_input_mappings.push(Some(position_in_stream));
                        }
                        SocketType::Midi => {
                            let position_in_midi = other_outputs
                                .midi_output_sockets
                                .iter()
                                .position(|&other_socket| other_socket == connection.data.from_socket)
                                .unwrap()
                                + other_outputs.midi_outputs_index;

                            self.midi_input_mappings.push(Some(position_in_midi));
                        }
                        SocketType::Value => {
                            let position_in_value = other_outputs
                                .value_output_sockets
                                .iter()
                                .position(|&other_socket| other_socket == connection.data.from_socket)
                                .unwrap()
                                + other_outputs.value_outputs_index;

                            self.value_input_mappings.push(Some(position_in_value));
                        }
                        SocketType::NodeRef => {}
                    }
                } else {
                    // it's not connected to anything, so push None in the mapping
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
        let nothing_in = vec![0.0_f32; self.buffer_size];

        let mut midi_mapping_i = 0;
        let mut value_mapping_i = 0;
        let mut stream_mapping_i = 0;

        let mut midi_inputs: [MaybeUninit<Option<MidiBundle>>; BUFFER_SIZE] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let mut value_inputs: [MaybeUninit<Option<Primitive>>; BUFFER_SIZE] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let mut stream_inputs: [MaybeUninit<&[f32]>; BUFFER_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };

        let mut midi_outputs_i = 0;
        let mut value_outputs_i = 0;
        let mut stream_outputs_i = 0;

        let mut stream_outputs: [MaybeUninit<&mut [f32]>; BUFFER_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };

        let mut requesting_graph_state = false;
        let mut requested_state_updates = vec![];

        // build the midi inputs and input
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let inputs = self.midi_advance_by[i].inputs;

            let mut should_input_midi = false;

            for j in 0..inputs {
                if let Some(input_index) = self.midi_input_mappings[midi_mapping_i] {
                    let incoming = self.midi_outputs[input_index].clone();
                    should_input_midi |= incoming.is_some();

                    midi_inputs[j].write(incoming);
                } else {
                    midi_inputs[j].write(None);
                }

                midi_mapping_i += 1;
            }

            if should_input_midi {
                // SAFETY: 0..inputs is initialized above
                node.node
                    .accept_midi_inputs(unsafe { mem::transmute::<_, &[Option<MidiBundle>]>(&midi_inputs[0..inputs]) });
            }
        }

        // build the value inputs and input
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let inputs = self.value_advance_by[i].inputs;

            let mut should_input_value = false;

            for j in 0..inputs {
                if let Some(input_index) = self.value_input_mappings[value_mapping_i] {
                    let incoming = self.value_outputs[input_index].clone();
                    should_input_value |= incoming.is_some();

                    value_inputs[j].write(incoming);
                } else {
                    value_inputs[j].write(None);
                }

                value_mapping_i += 1;
            }

            // override any values coming in with values from the user, if any
            for override_input in node.to_input.drain(..) {
                value_inputs[override_input.0].write(Some(override_input.1));
                should_input_value = true;
            }

            if should_input_value {
                // SAFETY: 0..inputs is initialized above
                node.node.accept_value_inputs(unsafe {
                    mem::transmute::<_, &[Option<Primitive>]>(&value_inputs[0..inputs])
                });
            }
        }

        for (node_index, new_node_state) in updated_node_states.into_iter() {
            let node = &mut self.nodes[self.node_to_location_mapping.get(&node_index).unwrap().vec_index];
            node.node.set_state(new_node_state);
        }

        for (i, (node, advance_by)) in self.nodes.iter_mut().zip(&self.stream_advance_by).enumerate() {
            let inputs = advance_by.inputs;
            let outputs = advance_by.outputs;

            let outputs_ptr = self.stream_outputs.as_mut_ptr();

            // pointer alias debugging
            // let mut alias_test = vec![false; self.stream_outputs.len()];

            // build the list of input references from other nodes' outputs
            for j in 0..inputs {
                if let Some(input_index) = self.stream_input_mappings[stream_mapping_i] {
                    // pointer alias debugging
                    //
                    // for i in output_index..(output_index + self.buffer_size) {
                    //     if alias_test[i] == true {
                    //         panic!("Aliasing at: {:?}", i);
                    //     }

                    //     alias_test[i] = true;
                    // }

                    // SAFETY: Make sure we don't have a slice exceed the length of the array
                    assert!(input_index + self.buffer_size <= self.stream_outputs.len());

                    stream_inputs[j]
                        .write(unsafe { slice::from_raw_parts(outputs_ptr.add(input_index), self.buffer_size) });
                } else {
                    stream_inputs[j].write(&nothing_in);
                }

                stream_mapping_i += 1;
            }

            // ...and the list of output references
            for j in 0..outputs {
                let output_index = stream_outputs_i + j * self.buffer_size;

                // pointer alias debugging
                //
                // for i in output_index..(output_index + self.buffer_size) {

                //     if alias_test[i] == true {
                //         panic!("Aliasing at: {}", i);
                //     }

                //     alias_test[i] = true;
                // }

                // SAFETY: Make sure we don't have a slice exceed the length of the array
                assert!(output_index + self.buffer_size <= self.stream_outputs.len());

                stream_outputs[j]
                    .write(unsafe { slice::from_raw_parts_mut(outputs_ptr.add(output_index), self.buffer_size) });
            }

            let res = node.node.process(
                NodeProcessState {
                    current_time,
                    script_engine,
                    resources,
                    state: StateInterface {
                        states: graph_state,
                        request_node_states: &mut || requesting_graph_state = true,
                        enqueue_state_updates: &mut |updates| requested_state_updates.extend(updates.into_iter()),
                    },
                },
                // SAFETY: we've already initialized 0..inputs and 0..outputs above
                unsafe { mem::transmute::<_, &[&[f32]]>(&stream_inputs[0..inputs]) },
                unsafe { mem::transmute::<_, &mut [&mut [f32]]>(&mut stream_outputs[0..outputs]) },
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

            stream_outputs_i += advance_by.outputs * self.buffer_size;
        }

        for (vec_index, node_index) in &self.nodes_linked_to_ui {
            if let Some(new_node_state) = self.nodes[*vec_index].node.get_state() {
                state_changes.push((*node_index, new_node_state));
            }
        }

        for (node, advance_by) in self.nodes.iter_mut().zip(&self.midi_advance_by) {
            let outputs = advance_by.outputs;
            let output_index = midi_outputs_i;

            // reset values back to None, they may be set after running `get_midi_outputs`
            self.midi_outputs[output_index..(output_index + outputs)].fill(None);
            node.node
                .get_midi_outputs(&mut self.midi_outputs[output_index..(output_index + outputs)]);

            midi_outputs_i += advance_by.outputs;
        }

        for (node, advance_by) in self.nodes.iter_mut().zip(&self.value_advance_by) {
            let outputs = advance_by.outputs;
            let output_index = value_outputs_i;

            // reset values back to None, they may be set after running `get_value_outputs`
            self.value_outputs[output_index..(output_index + outputs)].fill(None);
            node.node
                .get_value_outputs(&mut self.value_outputs[output_index..(output_index + outputs)]);

            value_outputs_i += advance_by.outputs;
        }

        for node in &mut self.nodes {
            node.node.finish();
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
        socket: Socket,
        value: Primitive,
    ) -> Result<(), NodeError> {
        let locations = self.node_to_location_mapping.get(&node_index);

        if let Some(locations) = locations {
            let value_index = locations
                .value_socket_to_index
                .iter()
                .find_map(|&(possible_socket, index)| if possible_socket == socket { Some(index) } else { None });

            if let Some(value_index) = value_index {
                self.nodes[locations.vec_index].to_input.push((value_index, value));

                Ok(())
            } else {
                Err(NodeError::SocketDoesNotExist { socket })
            }
        } else {
            Err(NodeError::NodeDoesNotExist { node_index })
        }
    }
}
