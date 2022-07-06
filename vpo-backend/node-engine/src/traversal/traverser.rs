use rhai::Engine;

use crate::{
    connection::{OutputSideConnection, SocketDirection, SocketType},
    errors::{ErrorsAndWarnings, NodeError, NodeWarning},
    graph::Graph,
    node::{NodeIndex, NodeRow},
};

use super::calculate_traversal_order::calculate_graph_traverse_order;

#[derive(Debug)]
struct NodeTraverseData {
    defaults_in: Vec<NodeRow>,
    outputs_to: Vec<OutputSideConnection>,
}

#[derive(Debug)]
pub struct Traverser {
    nodes: Vec<(NodeIndex, NodeTraverseData)>,
}

impl Traverser {
    pub fn get_traverser(graph: &Graph) -> Traverser {
        // first, get traversal order
        let traversal_order = calculate_graph_traverse_order(graph);

        let mut nodes: Vec<(NodeIndex, NodeTraverseData)> =
            Vec::with_capacity(traversal_order.len());

        for node_index in &traversal_order {
            let generational_node = graph.get_node(node_index).unwrap();

            let ref_to_node = generational_node.node;
            let node = (*ref_to_node).borrow();

            // make a list of all the socket defaults
            let defaults_list = node.get_node_rows().iter().filter_map(|row| {
                if let Some((socket_type, direction)) = row.clone().to_type_and_direction() {
                    if direction == SocketDirection::Input {
                        return Some(socket_type);
                    }
                }

                None
            });

            // populate the defaults for the traverser
            let defaults_in: Vec<NodeRow> = defaults_list
                .filter_map(|socket_type| node.get_default(&socket_type))
                .collect();

            // now, find where in the traversal order the linked nodes are
            let output_connections = node.get_output_connections().clone();

            let node_traverse_data = NodeTraverseData {
                defaults_in,
                outputs_to: output_connections,
            };

            nodes.push((node_index.clone(), node_traverse_data));
        }

        Traverser { nodes }
    }

    pub fn update_node_defaults(&mut self, graph: &Graph, node_index: &NodeIndex) {
        let node_wrapper_ref = graph.get_node(node_index).unwrap().node;
        let node_wrapper = (*node_wrapper_ref).borrow_mut();

        // redo the list of defaults for this node
        // make a list of all the socket defaults
        let defaults_list = node_wrapper.get_node_rows().iter().filter_map(|row| {
            if let Some((socket_type, direction)) = row.clone().to_type_and_direction() {
                if direction == SocketDirection::Input {
                    return Some(socket_type);
                }
            }

            None
        });

        // populate the defaults for the traverser
        let defaults_in: Vec<NodeRow> = defaults_list
            .filter_map(|socket_type| node_wrapper.get_default(&socket_type))
            .collect();

        println!("defaults: {:?}", defaults_in);

        if let Some(entry) = self.nodes.iter_mut().find(|entry| node_index == &entry.0) {
            entry.1.defaults_in = defaults_in;
        }
    }

    pub fn traverse(
        &self,
        graph: &mut Graph,
        input_defaults: bool,
        current_time: i64,
        scripting_engine: &Engine,
    ) -> Result<(), ErrorsAndWarnings> {
        let mut errors: Vec<NodeError> = vec![];
        let mut warnings: Vec<NodeWarning> = vec![];

        for (node_index, data) in &self.nodes {
            let node_wrapper_ref = graph.get_node(node_index).unwrap().node;
            let mut node_wrapper = (*node_wrapper_ref).borrow_mut();

            if input_defaults {
                for default in &data.defaults_in {
                    // println!("\n\nsending default: {:?}\n", default);
                    match default {
                        NodeRow::StreamInput(socket_type, default) => {
                            node_wrapper.accept_stream_input(socket_type, *default);
                        }
                        NodeRow::MidiInput(socket_type, default) => {
                            node_wrapper.accept_midi_input(socket_type, default.clone());
                        }
                        NodeRow::ValueInput(socket_type, default) => {
                            node_wrapper.accept_value_input(socket_type, default.clone());
                        }
                        NodeRow::NodeRefInput(_) => {}
                        _ => unreachable!(),
                    }
                }
            }

            // make de magic happenz
            let process_result = node_wrapper.process(current_time, scripting_engine);

            // record any errors
            if let Err(mut errors_and_warnings) = process_result {
                errors.append(&mut errors_and_warnings.errors);
                warnings.append(&mut errors_and_warnings.warnings);
            }

            for output_connection in &data.outputs_to {
                let other_node_wrapper_ref =
                    graph.get_node(&output_connection.to_node).unwrap().node;
                let mut other_node_wrapper = (*other_node_wrapper_ref).borrow_mut();

                match &output_connection.from_socket_type {
                    SocketType::Stream(stream_type) => {
                        let sample = node_wrapper.get_stream_output(stream_type);
                        other_node_wrapper.accept_stream_input(
                            &output_connection
                                .to_socket_type
                                .clone()
                                .as_stream()
                                .unwrap(),
                            sample,
                        );
                    }
                    SocketType::Midi(midi_type) => {
                        let midi = node_wrapper.get_midi_output(midi_type);

                        if !midi.is_empty() {
                            other_node_wrapper.accept_midi_input(
                                &output_connection.to_socket_type.clone().as_midi().unwrap(),
                                midi,
                            );
                        }
                    }
                    SocketType::Value(value_type) => {
                        let value = node_wrapper.get_value_output(value_type);

                        if let Some(value) = value {
                            other_node_wrapper.accept_value_input(
                                &output_connection.to_socket_type.clone().as_value().unwrap(),
                                value,
                            );
                        }
                    }
                    SocketType::NodeRef(_) => {}
                    SocketType::MethodCall(_) => todo!(),
                }
            }
        }

        if !errors.is_empty() || !warnings.is_empty() {
            Err(ErrorsAndWarnings { errors, warnings })
        } else {
            Ok(())
        }
    }
}
