use std::{
    collections::BTreeMap,
    iter::repeat,
    mem::{self, MaybeUninit},
    slice::{from_raw_parts, from_raw_parts_mut},
};

use arr_macro::arr;
use rhai::Engine;
use sound_engine::SoundConfig;

use crate::{
    connection::{MidiBundle, Primitive, Socket, SocketType},
    errors::{ErrorsAndWarnings, NodeError, NodeOk, NodeWarning},
    global_state::Resources,
    graph_manager::{GraphIndex, GraphManager},
    node::{NodeIndex, NodeInitState, NodeProcessState, NodeRow, NodeRuntime},
    nodes::variants::{new_variant, NodeVariant},
};

use super::calculate_traversal_order::calculate_graph_traverse_order;

const BUFFER_SIZE: usize = 128;

#[derive(Debug, Clone)]
struct AdvanceBy {
    pub inputs: usize,
    pub outputs: usize,
    pub defaults: usize,
}

#[derive(Debug, Clone)]
struct OutputLocations {
    pub stream_outputs_index: usize,
    pub stream_defaults_index: usize,
    pub stream_outputs: Vec<Socket>,
    pub midi_outputs_index: usize,
    pub midi_defaults_index: usize,
    pub midi_outputs: Vec<Socket>,
    pub value_outputs_index: usize,
    pub value_defaults_index: usize,
    pub value_outputs: Vec<Socket>,
}

#[derive(Debug, Clone, Default)]
pub struct BufferedTraverser {
    buffer_size: usize,
    nodes: Vec<NodeVariant>,
    node_indexes: Vec<NodeIndex>,
    node_to_location_mapping: BTreeMap<NodeIndex, OutputLocations>,
    stream_outputs: Vec<f32>,
    stream_input_mappings: Vec<usize>,
    stream_advance_by: Vec<AdvanceBy>,
    midi_outputs: Vec<Option<MidiBundle>>,
    midi_input_mappings: Vec<usize>,
    midi_advance_by: Vec<AdvanceBy>,
    value_outputs: Vec<Option<Primitive>>,
    value_input_mappings: Vec<usize>,
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
    // outputs: [node1_defaults]|[node1_outputs]|[node2_defaults]|[node2_ouputs]...
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

        let graph = graph_manager.get_graph(graph_index)?.graph.borrow();

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

        let mut errors: Vec<(NodeIndex, NodeError)> = vec![];
        let mut warnings: Vec<(NodeIndex, NodeWarning)> = vec![];

        // now for the fun part
        for index in &traversal_order {
            // create and init the node
            let node_wrapper = graph.get_node(*index)?;

            let mut variant = if let Some(previous_node) = old_nodes.remove(index) {
                previous_node
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
                },
                child_graph_info,
            );

            // handle any errors from initializing the node
            match init_result_res {
                Ok(init_result) => {
                    for warning in init_result.warnings.into_iter() {
                        warnings.push((*index, warning))
                    }
                }
                Err(err) => {
                    errors.push((*index, err));
                }
            };

            // create a list of its default inputs and count the outputs
            let mut needed_stream_defaults = vec![];
            let mut needed_midi_defaults = vec![];
            let mut needed_value_defaults = vec![];

            let mut stream_inputs = 0;
            let mut midi_inputs = 0;
            let mut value_inputs = 0;

            let mut stream_outputs = vec![];
            let mut midi_outputs = vec![];
            let mut value_outputs = vec![];

            for socket in node_wrapper.list_input_sockets() {
                let default_row = node_wrapper.get_default(socket).unwrap();

                if let NodeRow::Input(socket, default) = default_row {
                    let is_connected = graph.get_input_connection_index(*index, socket)?.is_some();

                    match socket.socket_type() {
                        SocketType::Stream => {
                            if !is_connected {
                                needed_stream_defaults.push(default.as_stream().unwrap());
                            }

                            stream_inputs += 1;
                        }
                        SocketType::Midi => {
                            if !is_connected {
                                needed_midi_defaults.push(default.as_midi().unwrap());
                            }

                            midi_inputs += 1;
                        }
                        SocketType::Value => {
                            if !is_connected {
                                needed_value_defaults.push(default.as_value().unwrap());
                            }

                            value_inputs += 1;
                        }
                        _ => {}
                    }
                }
            }

            for socket in node_wrapper.list_output_sockets() {
                match socket.socket_type() {
                    SocketType::Stream => stream_outputs.push(socket),
                    SocketType::Midi => midi_outputs.push(socket),
                    SocketType::Value => value_outputs.push(socket),
                    _ => {}
                }
            }

            self.nodes.push(variant);
            self.node_indexes.push(*index);

            let stream_defaults_index = self.stream_outputs.len();
            let midi_defaults_index = self.midi_outputs.len();
            let value_defaults_index = self.value_outputs.len();

            // defaults are stored right before the node's outputs
            for default in &needed_stream_defaults {
                self.stream_outputs.extend(repeat(default).take(self.buffer_size));
            }

            for default in &needed_midi_defaults {
                self.midi_outputs.push(Some(default.clone()));
            }

            for default in &needed_value_defaults {
                self.value_outputs.push(Some(default.clone()));
            }

            self.node_to_location_mapping.insert(
                *index,
                OutputLocations {
                    stream_outputs_index: self.stream_outputs.len(),
                    stream_defaults_index,
                    stream_outputs: stream_outputs.clone(),
                    midi_outputs_index: self.midi_outputs.len(),
                    midi_defaults_index,
                    midi_outputs: midi_outputs.clone(),
                    value_outputs_index: self.value_outputs.len(),
                    value_defaults_index,
                    value_outputs: value_outputs.clone(),
                },
            );

            // figure out how much the traverser needs to advance between each node
            self.stream_advance_by.push(AdvanceBy {
                inputs: stream_inputs,
                outputs: stream_outputs.len(),
                defaults: needed_stream_defaults.len(),
            });

            self.midi_advance_by.push(AdvanceBy {
                inputs: midi_inputs,
                outputs: midi_outputs.len(),
                defaults: needed_midi_defaults.len(),
            });

            self.value_advance_by.push(AdvanceBy {
                inputs: value_inputs,
                outputs: value_outputs.len(),
                defaults: needed_value_defaults.len(),
            });

            self.stream_outputs
                .extend(repeat(0.0).take(stream_outputs.len() * self.buffer_size));
            self.midi_outputs.extend(repeat(None).take(midi_outputs.len()));
            self.value_outputs.extend(repeat(None).take(value_outputs.len()));
        }

        // the next step is to populate the input mappings, since we know where all the nodes are now
        // The input mappings is a mapping to get the node's next input
        for index in traversal_order.iter() {
            let wrapper = graph.get_node(*index)?;

            let mut stream_default_at = 0;
            let mut midi_default_at = 0;
            let mut value_default_at = 0;

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
                                .stream_outputs
                                .iter()
                                .position(|&other_socket| other_socket == connection.data.from_socket)
                                .unwrap()
                                * self.buffer_size
                                + other_outputs.stream_outputs_index;

                            self.stream_input_mappings.push(position_in_stream);
                        }
                        SocketType::Midi => {
                            let position_in_midi = other_outputs
                                .midi_outputs
                                .iter()
                                .position(|&other_socket| other_socket == connection.data.from_socket)
                                .unwrap()
                                + other_outputs.midi_outputs_index;

                            self.midi_input_mappings.push(position_in_midi);
                        }
                        SocketType::Value => {
                            let position_in_value = other_outputs
                                .value_outputs
                                .iter()
                                .position(|&other_socket| other_socket == connection.data.from_socket)
                                .unwrap()
                                + other_outputs.value_outputs_index;

                            self.value_input_mappings.push(position_in_value);
                        }
                        SocketType::NodeRef => {}
                    }
                } else {
                    // it's not connected to anything, so point it to its default
                    match input.socket_type() {
                        SocketType::Stream => {
                            self.stream_input_mappings.push(
                                self.node_to_location_mapping.get(index).unwrap().stream_defaults_index
                                    + stream_default_at * self.buffer_size,
                            );

                            stream_default_at += 1;
                        }
                        SocketType::Midi => {
                            self.midi_input_mappings.push(
                                self.node_to_location_mapping.get(index).unwrap().midi_defaults_index + midi_default_at,
                            );

                            midi_default_at += 1;
                        }
                        SocketType::Value => {
                            self.value_input_mappings.push(
                                self.node_to_location_mapping.get(index).unwrap().value_defaults_index
                                    + value_default_at,
                            );

                            value_default_at += 1;
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

    pub fn traverse(&mut self, current_time: i64, script_engine: &Engine, resources: &Resources) -> ErrorsAndWarnings {
        let mut errors: Vec<(NodeIndex, NodeError)> = vec![];
        let mut warnings: Vec<(NodeIndex, NodeWarning)> = vec![];

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

        // build the midi inputs and input
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let inputs = self.midi_advance_by[i].inputs;

            let mut should_input_midi = false;

            for j in 0..inputs {
                let incoming = self.midi_outputs[self.midi_input_mappings[midi_mapping_i]].clone();
                should_input_midi |= incoming.is_some();

                midi_inputs[j].write(incoming);
                midi_mapping_i += 1;
            }

            if should_input_midi {
                node.accept_midi_inputs(unsafe { mem::transmute::<_, &[Option<MidiBundle>]>(&midi_inputs[0..inputs]) });
            }
        }

        // build the value inputs and input
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let inputs = self.value_advance_by[i].inputs;

            let mut should_input_value = false;

            for j in 0..inputs {
                let incoming = self.value_outputs[self.value_input_mappings[value_mapping_i]].clone();
                should_input_value |= incoming.is_some();

                value_inputs[j].write(incoming);
                value_mapping_i += 1;
            }

            if should_input_value {
                node.accept_value_inputs(unsafe {
                    mem::transmute::<_, &[Option<Primitive>]>(&value_inputs[0..inputs])
                });
            }
        }

        for (i, (node, advance_by)) in self.nodes.iter_mut().zip(&self.stream_advance_by).enumerate() {
            let inputs = advance_by.inputs;
            let outputs = advance_by.outputs;

            let outputs_ptr = self.stream_outputs.as_mut_ptr();

            // aliasing testing
            // let mut alias_test = vec![false; self.stream_outputs.len()];

            // build the list of input references from other nodes' outputs
            for j in 0..inputs {
                let output_index = self.stream_input_mappings[stream_mapping_i];

                // alias testing
                // assert!(output_index + self.buffer_size <= self.stream_outputs.len());
                //
                // for i in output_index..(output_index + self.buffer_size) {
                //     if alias_test[i] == true {
                //         panic!("Aliasing at: {:?}", i);
                //     }

                //     alias_test[i] = true;
                // }

                unsafe {
                    stream_inputs[j].write(from_raw_parts(outputs_ptr.add(output_index), self.buffer_size));
                }

                stream_mapping_i += 1;

                // ...and the list of output references
                for j in 0..outputs {
                    let output_index = stream_outputs_i + (advance_by.defaults + j) * self.buffer_size;

                    // alias test
                    // assert!(output_index + self.buffer_size <= self.stream_outputs.len());
                    //
                    // for i in output_index..(output_index + self.buffer_size) {

                    //     if alias_test[i] == true {
                    //         panic!("Aliasing at: {}", i);
                    //     }

                    //     alias_test[i] = true;
                    // }

                    unsafe {
                        stream_outputs[j].write(from_raw_parts_mut(outputs_ptr.add(output_index), self.buffer_size));
                    }
                }
            }

            let res = node.process(
                NodeProcessState {
                    current_time,
                    script_engine,
                    resources,
                },
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

            stream_outputs_i += (advance_by.defaults + advance_by.outputs) * self.buffer_size;
        }

        for (node, advance_by) in self.nodes.iter_mut().zip(&self.midi_advance_by) {
            let outputs = advance_by.outputs;
            let output_index = midi_outputs_i + advance_by.defaults;

            // reset values back to None
            self.midi_outputs[output_index..(output_index + outputs)].fill(None);
            node.get_midi_outputs(&mut self.midi_outputs[output_index..(output_index + outputs)]);

            midi_outputs_i += advance_by.defaults + advance_by.outputs;
        }

        for (node, advance_by) in self.nodes.iter_mut().zip(&self.value_advance_by) {
            let outputs = advance_by.outputs;
            let output_index = value_outputs_i + advance_by.defaults;

            // reset values back to None
            self.value_outputs[output_index..(output_index + outputs)].fill(None);
            node.get_value_outputs(&mut self.value_outputs[output_index..(output_index + outputs)]);

            value_outputs_i += advance_by.defaults + advance_by.outputs;
        }

        ErrorsAndWarnings { errors, warnings }
    }

    pub fn get_node_mut(&mut self, index_to_find: NodeIndex) -> Option<&mut NodeVariant> {
        self.nodes
            .iter_mut()
            .zip(&self.node_indexes)
            .find(|(_, index)| *index == &index_to_find)
            .map(|(node, _)| node)
    }
}
