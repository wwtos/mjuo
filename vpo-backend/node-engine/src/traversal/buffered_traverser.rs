use std::{collections::HashMap, iter::repeat};

use arr_macro::arr;
use rhai::Engine;
use web_sys::console;

use crate::{
    connection::{MidiBundle, Primitive, Socket, SocketType},
    errors::{ErrorsAndWarnings, NodeError, Warnings},
    global_state::GlobalState,
    graph_manager::{GraphIndex, GraphManager},
    node::{NodeGraphAndIo, NodeIndex, NodeInitState, NodeProcessState, NodeRow, NodeRuntime},
    nodes::variants::{new_variant, NodeVariant},
};

use super::calculate_traversal_order::calculate_graph_traverse_order;

struct AdvanceBy {
    pub inputs: usize,
    pub outputs: usize,
    pub defaults: usize,
}

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

pub struct Traverser {
    buffer_size: usize,
    nodes: Vec<NodeVariant>,
    node_indexes: Vec<NodeIndex>,
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

impl Traverser {
    pub fn init_graph(
        &mut self,
        graph_index: GraphIndex,
        graph_manager: &GraphManager,
        script_engine: &Engine,
        global_state: &GlobalState,
        current_time: i64,
        buffer_size: usize,
    ) -> Result<(), NodeError> {
        self.buffer_size = buffer_size;

        let graph = graph_manager.get_graph(graph_index)?.graph.borrow();

        // figure out what order we should go through the nodes
        let traversal_order = calculate_graph_traverse_order(&graph);

        // pull out the old nodes (don't recreate them every time)
        let mut old_nodes = self.nodes.drain(0..).zip(self.node_indexes.drain(0..)).fold(
            HashMap::new(),
            |mut map, (node, node_index)| {
                map.insert(node_index, node);

                map
            },
        );

        self.nodes.clear();
        self.node_indexes.clear();

        let mut errors: Vec<(NodeIndex, NodeError)> = vec![];
        let mut warnings: Vec<(NodeIndex, Warnings)> = Vec::new();

        let mut node_to_location_mapping: HashMap<NodeIndex, OutputLocations> = HashMap::new();

        // now for the fun part
        for index in &traversal_order {
            // create and init the node
            let node_wrapper = graph.get_node(*index)?;

            let mut variant = if let Some(previous_node) = old_nodes.remove(&index) {
                previous_node
            } else {
                new_variant(&node_wrapper.get_node_type(), &global_state.sound_config)?
            };

            // extract the graph and child io indexes, if any
            let child_graph_info = node_wrapper
                .get_child_graph_info()
                .map(|(graph_index, child_io_indexes)| NodeGraphAndIo {
                    graph: graph_index,
                    input_index: child_io_indexes.0,
                    output_index: child_io_indexes.1,
                });

            let init_result_res = variant.init(
                NodeInitState {
                    props: node_wrapper.get_properties(),
                    script_engine,
                    global_state,
                    current_time,
                    graph_manager,
                },
                child_graph_info,
            );

            // handle any errors from initializing the node
            match init_result_res {
                Ok(init_result) => {
                    if let Some(new_warnings) = init_result.warnings {
                        warnings.push((*index, new_warnings))
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
            for default in needed_stream_defaults {
                self.stream_outputs.extend(repeat(default).take(buffer_size));
            }

            for default in needed_midi_defaults {
                self.midi_outputs.push(Some(default));
            }

            for default in needed_value_defaults {
                self.value_outputs.push(Some(default));
            }

            node_to_location_mapping.insert(
                *index,
                OutputLocations {
                    stream_outputs_index: self.stream_outputs.len(),
                    stream_defaults_index,
                    stream_outputs,
                    midi_outputs_index: self.midi_outputs.len(),
                    midi_defaults_index,
                    midi_outputs,
                    value_outputs_index: self.value_outputs.len(),
                    value_defaults_index,
                    value_outputs,
                },
            );

            for default in needed_stream_defaults {
                self.stream_outputs.extend(repeat(default).take(buffer_size));
            }

            self.midi_outputs
                .extend(needed_midi_defaults.iter().cloned().map(|x| Some(x)));
            self.value_outputs
                .extend(needed_value_defaults.iter().cloned().map(|x| Some(x)));

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
                .extend(repeat(0.0).take(stream_outputs.len() * buffer_size));
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
                    let other_outputs = node_to_location_mapping.get(&from).unwrap();

                    // add it to the mapping
                    match input.socket_type() {
                        SocketType::Stream => {
                            let position_in_stream = other_outputs
                                .stream_outputs
                                .iter()
                                .position(|&other_socket| other_socket == connection.data.from_socket)
                                .unwrap()
                                * buffer_size
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
                                node_to_location_mapping.get(index).unwrap().stream_defaults_index
                                    + stream_default_at * buffer_size,
                            );

                            stream_default_at += 1;
                        }
                        SocketType::Midi => {
                            self.midi_input_mappings.push(
                                node_to_location_mapping.get(index).unwrap().midi_defaults_index + midi_default_at,
                            );

                            midi_default_at += 1;
                        }
                        SocketType::Value => {
                            self.value_input_mappings.push(
                                node_to_location_mapping.get(index).unwrap().value_defaults_index + value_default_at,
                            );

                            value_default_at += 1;
                        }
                        SocketType::NodeRef => {}
                    }
                }
            }
        }

        if !errors.is_empty() {
            console::log_1(&format!("errors: {:#?}", errors).into());
        }

        Ok(())
    }

    pub fn traverse(
        &mut self,
        current_time: i64,
        script_engine: &Engine,
        global_state: &GlobalState,
        out: &mut [f32],
    ) -> Result<(), ErrorsAndWarnings> {
        assert_eq!(out.len(), self.buffer_size);

        let mut midi_mapping_i = 0;
        let mut value_mapping_i = 0;
        let mut stream_mapping_i = 0;

        let mut midi_inputs: [Option<MidiBundle>; 128] = arr![None; 128];
        let mut value_inputs: [Option<Primitive>; 128] = arr![None; 128];
        let mut stream_inputs: Vec<&[f32]> = Vec::with_capacity(128);

        let mut midi_outputs_i = 0;
        let mut value_outputs_i = 0;
        let mut stream_outputs_i = 0;

        let mut midi_outputs: [Option<MidiBundle>; 128] = arr![None; 128];
        let mut value_outputs: [Option<Primitive>; 128] = arr![None; 128];
        let mut stream_outputs: Vec<&mut [f32]> = Vec::with_capacity(128);

        for frame in out {
            *frame = 0.0;
        }

        for (i, node) in self.nodes.iter_mut().enumerate() {
            let inputs = self.midi_advance_by[i].inputs;

            let mut should_input_midi = false;

            for j in 0..inputs {
                midi_inputs[j] = self.midi_outputs[self.midi_input_mappings[midi_mapping_i]];
                midi_mapping_i += 1;

                should_input_midi |= midi_inputs[j].is_some();
            }

            if should_input_midi {
                node.accept_midi_inputs(&midi_inputs[0..inputs]);
            }
        }

        for (i, node) in self.nodes.iter_mut().enumerate() {
            let inputs = self.value_advance_by[i].inputs;

            let mut should_input_value = false;

            for j in 0..inputs {
                value_inputs[j] = self.value_outputs[self.value_input_mappings[value_mapping_i]];
                value_mapping_i += 1;

                should_input_value |= value_inputs[j].is_some();
            }

            if should_input_value {
                node.accept_value_inputs(&value_inputs[0..inputs]);
            }
        }

        for (node, advance_by) in self.nodes.iter_mut().zip(self.stream_advance_by) {
            let inputs = advance_by.inputs;
            let outputs = advance_by.outputs;

            // build the list of inputs
            for j in 0..inputs {
                let output_index = self.stream_input_mappings[stream_mapping_i];

                if stream_inputs.len() < j {
                    stream_inputs.push(&self.stream_outputs[output_index..(output_index + self.buffer_size)]);
                } else {
                    stream_inputs[j] = &self.stream_outputs[output_index..(output_index + self.buffer_size)];
                }

                stream_mapping_i += 1;
            }

            // ...and the list of outputs
            for j in 0..outputs {
                let output_index = stream_outputs_i + advance_by.defaults + j * self.buffer_size;

                if stream_outputs.len() < j {
                    stream_outputs.push(&mut self.stream_outputs[output_index..(output_index + self.buffer_size)]);
                } else {
                    stream_outputs[j] = &mut self.stream_outputs[output_index..(output_index + self.buffer_size)];
                }
            }

            let res = node.process(
                NodeProcessState {
                    current_time,
                    script_engine,
                    global_state,
                },
                &stream_inputs[0..inputs],
                &mut stream_outputs[0..outputs],
            );

            stream_outputs_i += (advance_by.defaults + advance_by.outputs) * self.buffer_size;
        }

        for (node, advance_by) in self.nodes.iter_mut().zip(self.midi_advance_by) {
            let outputs = advance_by.outputs;
            let output_index = midi_outputs_i + advance_by.defaults;

            node.get_midi_outputs(&mut self.midi_outputs[output_index..(output_index + outputs)]);

            midi_outputs_i += advance_by.defaults + advance_by.outputs;
        }

        for (node, advance_by) in self.nodes.iter_mut().zip(self.value_advance_by) {
            let outputs = advance_by.outputs;
            let output_index = value_outputs_i + advance_by.defaults;

            node.get_value_outputs(&mut self.value_outputs[output_index..(output_index + outputs)]);

            value_outputs_i += advance_by.defaults + advance_by.outputs;
        }

        Ok(())
    }
}
