use rhai::Engine;

use crate::{
    connection::{OutputSideConnection, SocketDirection, SocketType},
    errors::{ErrorsAndWarnings, NodeError, NodeWarning},
    global_state::GlobalState,
    node::{NodeIndex, NodeProcessState, NodeRow},
    node_graph::NodeGraph,
};

use super::calculate_traversal_order::calculate_graph_traverse_order;

#[derive(Debug, Clone)]
struct NodeTraverseData {
    defaults_in: Vec<NodeRow>,
    outputs_to: Vec<OutputSideConnection>,
}

#[derive(Debug, Clone)]
pub struct Traverser {
    nodes: Vec<(NodeIndex, NodeTraverseData)>,
}

impl Default for Traverser {
    fn default() -> Self {
        Self::new()
    }
}

impl Traverser {
    pub fn new() -> Self {
        Traverser { nodes: vec![] }
    }

    pub fn get_traverser(graph: &NodeGraph) -> Traverser {
        // first, get traversal order
        let traversal_order = calculate_graph_traverse_order(graph);

        let mut nodes: Vec<(NodeIndex, NodeTraverseData)> = Vec::with_capacity(traversal_order.len());

        for node_index in &traversal_order {
            let node = graph.get_node(node_index).unwrap();

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

            nodes.push((*node_index, node_traverse_data));
        }

        Traverser { nodes }
    }

    pub fn update_node_defaults(&mut self, graph: &NodeGraph, node_index: &NodeIndex) {
        let node_wrapper = graph.get_node(node_index).unwrap();

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

        if let Some(entry) = self.nodes.iter_mut().find(|entry| node_index == &entry.0) {
            entry.1.defaults_in = defaults_in;
        }
    }

    pub fn traverse(
        &self,
        graph: &mut NodeGraph,
        input_defaults: bool,
        current_time: i64,
        script_engine: &Engine,
        global_state: &GlobalState,
    ) -> Result<(), ErrorsAndWarnings> {
        let mut errors: Vec<NodeError> = vec![];
        let mut warnings: Vec<NodeWarning> = vec![];

        for (node_index, data) in &self.nodes {
            let node_wrapper = graph.get_node_mut(node_index).unwrap();

            if input_defaults {
                for default in &data.defaults_in {
                    // println!("\n\nsending default: {:?}\n", default);
                    match default {
                        NodeRow::StreamInput(socket_type, default, _) => {
                            node_wrapper.accept_stream_input(socket_type, *default);
                        }
                        NodeRow::MidiInput(socket_type, default, _) => {
                            node_wrapper.accept_midi_input(socket_type, default.clone());
                        }
                        NodeRow::ValueInput(socket_type, default, _) => {
                            node_wrapper.accept_value_input(socket_type, default.clone());
                        }
                        NodeRow::NodeRefInput(..) => {}
                        _ => unreachable!(),
                    }
                }
            }

            // make de magic happenz
            // TODO: Don't just input 'none' into the graph
            let process_result = node_wrapper.process(NodeProcessState {
                current_time,
                script_engine,
                child_graph: None,
                global_state,
            });

            // record any errors
            match process_result {
                Ok(result) => {
                    if let Some(new_warnings) = result.warnings {
                        warnings.extend(new_warnings.warnings)
                    }
                }
                Err(err) => {
                    errors.push(err);
                }
            }

            for output_connection in &data.outputs_to {
                match &output_connection.from_socket_type {
                    SocketType::Stream(stream_type) => {
                        let node_wrapper = graph.get_node_mut(node_index).unwrap();
                        let sample = node_wrapper.get_stream_output(stream_type);

                        let other_node_wrapper = graph.get_node_mut(&output_connection.to_node).unwrap();
                        other_node_wrapper.accept_stream_input(
                            &output_connection.to_socket_type.clone().as_stream().unwrap(),
                            sample,
                        );
                    }
                    SocketType::Midi(midi_type) => {
                        let node_wrapper = graph.get_node_mut(node_index).unwrap();
                        let midi = node_wrapper.get_midi_output(midi_type);

                        let other_node_wrapper = graph.get_node_mut(&output_connection.to_node).unwrap();
                        if let Some(midi) = midi {
                            other_node_wrapper
                                .accept_midi_input(&output_connection.to_socket_type.clone().as_midi().unwrap(), midi);
                        }
                    }
                    SocketType::Value(value_type) => {
                        let node_wrapper = graph.get_node_mut(node_index).unwrap();
                        let value = node_wrapper.get_value_output(value_type);

                        let other_node_wrapper = graph.get_node_mut(&output_connection.to_node).unwrap();
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
