use std::collections::HashMap;

use arr_macro::arr;
use ddgg::Edge;
use rhai::Engine;
use web_sys::console;

use crate::{
    connection::{MidiBundle, Primitive, SocketType},
    errors::{ErrorsAndWarnings, NodeError, WarningBuilder},
    global_state::GlobalState,
    graph_manager::{GraphIndex, GraphManager},
    node::{NodeGraphAndIo, NodeIndex, NodeInitState, NodeProcessState, NodeRow, NodeRuntime},
    node_graph::NodeConnection,
    nodes::variants::{new_variant, NodeVariant},
};

use super::calculate_traversal_order::calculate_graph_traverse_order;

#[derive(Debug, Clone)]
struct NodeState {
    node: NodeVariant,
    node_index: NodeIndex,
    stream_index: usize,
    midi_index: usize,
    value_index: usize,
    linked_to_ui: bool,
    stream_input_count: usize,
    stream_output_count: usize,
    stream_output_mappings: usize,
    midi_input_count: usize,
    midi_output_count: usize,
    midi_output_mappings: usize,
    value_input_count: usize,
    value_output_count: usize,
    value_output_mappings: usize,
}

#[derive(Debug, Clone)]
pub struct RealtimeTraverser {
    nodes: Vec<NodeState>,
    stream_inputs: Vec<f32>,
    stream_output_mappings: Vec<(usize, usize)>,
    midi_inputs: Vec<Option<MidiBundle>>,
    midi_output_mappings: Vec<(usize, usize)>,
    value_inputs: Vec<Option<Primitive>>,
    value_output_mappings: Vec<(usize, usize)>,
    midi_default_indexes: Vec<usize>,
    value_default_indexes: Vec<usize>,
    reset_needed: bool,
}

impl Default for RealtimeTraverser {
    fn default() -> Self {
        Self::new()
    }
}

impl RealtimeTraverser {
    pub fn new() -> Self {
        RealtimeTraverser {
            nodes: vec![],
            stream_inputs: vec![],
            stream_output_mappings: vec![],
            midi_inputs: vec![],
            midi_output_mappings: vec![],
            value_inputs: vec![],
            value_output_mappings: vec![],
            midi_default_indexes: vec![],
            value_default_indexes: vec![],
            reset_needed: true,
        }
    }

    pub fn get_traverser(
        graph_index: GraphIndex,
        graph_manager: &GraphManager,
        script_engine: &Engine,
        global_state: &GlobalState,
        current_time: i64,
    ) -> Result<RealtimeTraverser, NodeError> {
        let mut traverser = RealtimeTraverser::new();

        traverser
            .init_graph(graph_index, graph_manager, script_engine, global_state, current_time)
            .map(|()| traverser)
    }

    pub fn init_graph(
        &mut self,
        graph_index: GraphIndex,
        graph_manager: &GraphManager,
        script_engine: &Engine,
        global_state: &GlobalState,
        current_time: i64,
    ) -> Result<(), NodeError> {
        let graph = graph_manager.get_graph(graph_index)?.graph.borrow();
        let traversal_order = calculate_graph_traverse_order(&graph);

        let mut old_nodes = self.nodes.drain(0..).fold(HashMap::new(), |mut map, node| {
            map.insert(node.node_index, node.node);

            map
        });

        self.nodes.clear();
        self.stream_inputs.clear();
        self.stream_output_mappings.clear();
        self.midi_inputs.clear();
        self.midi_output_mappings.clear();
        self.value_inputs.clear();
        self.value_output_mappings.clear();
        self.midi_default_indexes.clear();
        self.value_default_indexes.clear();

        let mut errors: Vec<(NodeIndex, NodeError)> = vec![];
        let mut warnings: WarningBuilder = WarningBuilder::new();

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

            match init_result_res {
                Ok(init_result) => warnings.append_warnings(init_result.warnings),
                Err(err) => {
                    errors.push((*index, err));
                }
            };

            // create a list of its default inputs and count the outputs
            let mut stream_input_defaults = vec![];
            let mut midi_input_defaults = vec![];
            let mut value_input_defaults = vec![];

            let mut stream_outputs = 0;
            let mut midi_outputs = 0;
            let mut value_outputs = 0;

            for socket_type in node_wrapper.list_input_sockets() {
                let default_row = node_wrapper.get_default(socket_type).unwrap();

                if let NodeRow::Input(socket, default) = default_row {
                    match socket.socket_type() {
                        SocketType::Stream => stream_input_defaults.push(default.as_stream().unwrap()),
                        SocketType::Midi => midi_input_defaults.push(default.as_midi().unwrap()),
                        SocketType::Value => value_input_defaults.push(default.as_value().unwrap()),
                        _ => {}
                    }
                }
            }

            for socket in node_wrapper.list_output_sockets() {
                match socket.socket_type() {
                    SocketType::Stream => stream_outputs += 1,
                    SocketType::Midi => midi_outputs += 1,
                    SocketType::Value => value_outputs += 1,
                    _ => {}
                }
            }

            let linked_to_ui = variant.linked_to_ui();

            // next, add the node
            let new_node = NodeState {
                node: variant,
                node_index: *index,
                stream_index: self.stream_inputs.len(),
                midi_index: self.midi_inputs.len(),
                value_index: self.value_inputs.len(),
                linked_to_ui,
                stream_input_count: stream_input_defaults.len(),
                stream_output_count: stream_outputs,
                stream_output_mappings: 0,
                midi_input_count: midi_input_defaults.len(),
                midi_output_count: midi_outputs,
                midi_output_mappings: 0,
                value_input_count: value_input_defaults.len(),
                value_output_count: value_outputs,
                value_output_mappings: 0,
            };

            self.nodes.push(new_node);

            // note the indexes of the defaults that need to be reset (so they aren't inputted
            // every time)
            let inputs = node_wrapper.list_input_sockets();
            let connected = graph.get_input_side_connections(*index)?;

            let mut midi_index = 0;
            let mut value_index = 0;

            for input in inputs {
                if !connected.iter().any(|connection| connection.to_socket == input) {
                    match input.socket_type() {
                        SocketType::Midi => {
                            self.midi_default_indexes.push(self.midi_inputs.len() + midi_index);
                        }
                        SocketType::Value => {
                            self.value_default_indexes.push(self.value_inputs.len() + value_index);
                        }
                        SocketType::NodeRef | SocketType::Stream => {}
                    }
                }

                match input.socket_type() {
                    SocketType::Midi => midi_index += 1,
                    SocketType::Value => value_index += 1,
                    SocketType::NodeRef | SocketType::Stream => {}
                }
            }

            // populate the inputs with the defaults (which will be read every time)
            self.stream_inputs.extend(stream_input_defaults);
            self.midi_inputs
                .extend(midi_input_defaults.into_iter().map(|x| Some(x)));
            self.value_inputs
                .extend(value_input_defaults.into_iter().map(|x| Some(x)));
        }

        // now that we know the indexes of all the nodes, we can populate the output mappings
        for index in traversal_order.iter() {
            let node_wrapper = graph.get_node(*index)?;

            let mut stream_index = 0;
            let mut midi_index = 0;
            let mut value_index = 0;

            let stream_mapping_len = self.stream_output_mappings.len();
            let midi_mapping_len = self.midi_output_mappings.len();
            let value_mapping_len = self.value_output_mappings.len();

            // where does this node connect to?
            for socket in node_wrapper.list_output_sockets() {
                let connection_indexes = graph.get_output_connection_indexes(*index, socket);

                if let Ok(indexes) = connection_indexes {
                    let connections: Vec<&Edge<NodeConnection>> = indexes
                        .iter()
                        .map(|index| graph.get_graph().get_edge(index.0).unwrap())
                        .collect();

                    for connection in connections {
                        let to_index = NodeIndex(connection.get_to());
                        let to = graph.get_graph().get_vertex_data(connection.get_to())?;

                        let mut other_stream_index = 0;
                        let mut other_midi_index = 0;
                        let mut other_value_index = 0;

                        // find this input on the "to" node, and get its index
                        for other_input_socket in to.list_input_sockets() {
                            if other_input_socket == connection.data.to_socket {
                                let to_local_node = self.nodes.iter().find(|x| x.node_index == to_index).unwrap();

                                match socket.socket_type() {
                                    SocketType::Stream => self
                                        .stream_output_mappings
                                        .push((stream_index, to_local_node.stream_index + other_stream_index)),
                                    SocketType::Midi => self
                                        .midi_output_mappings
                                        .push((midi_index, to_local_node.midi_index + other_midi_index)),
                                    SocketType::Value => self
                                        .value_output_mappings
                                        .push((value_index, to_local_node.value_index + other_value_index)),
                                    _ => {}
                                }

                                break;
                            }

                            match socket.socket_type() {
                                SocketType::Stream => other_stream_index += 1,
                                SocketType::Midi => other_midi_index += 1,
                                SocketType::Value => other_value_index += 1,
                                _ => {}
                            }
                        }
                    }
                }

                match socket.socket_type() {
                    SocketType::Stream => stream_index += 1,
                    SocketType::Midi => midi_index += 1,
                    SocketType::Value => value_index += 1,
                    _ => {}
                }
            }

            let local_node = self.nodes.iter_mut().find(|node| node.node_index == *index).unwrap();

            local_node.stream_output_mappings = self.stream_output_mappings.len() - stream_mapping_len;
            local_node.midi_output_mappings = self.midi_output_mappings.len() - midi_mapping_len;
            local_node.value_output_mappings = self.value_output_mappings.len() - value_mapping_len;
        }

        if !errors.is_empty() {
            console::log_1(&format!("errors: {:#?}", errors).into());
        }

        // console::log_1(&format!("traverser: {:#?}", self).into());

        self.reset_needed = true;

        Ok(())
    }

    pub fn traverse(
        &mut self,
        current_time: i64,
        script_engine: &Engine,
        global_state: &GlobalState,
    ) -> Result<(), ErrorsAndWarnings> {
        let mut stream_output_mappings_i = 0;
        let mut midi_output_mappings_i = 0;
        let mut value_output_mappings_i = 0;

        let mut errors: Vec<(NodeIndex, NodeError)> = vec![];
        let mut warnings: WarningBuilder = WarningBuilder::new();

        let mut stream_staging: [f32; 64] = [0.0; 64];
        let mut midi_staging: [Option<MidiBundle>; 64] = arr![None; 64];
        let mut value_staging: [Option<Primitive>; 64] = arr![None; 64];

        for node in &mut self.nodes {
            let stream_input_index = node.stream_index;
            let midi_input_index = node.midi_index;
            let value_input_index = node.value_index;

            // get input slices
            let midi_inputs = &self.midi_inputs[midi_input_index..(midi_input_index + node.midi_input_count)];
            let value_inputs = &self.value_inputs[value_input_index..(value_input_index + node.value_input_count)];
            let stream_inputs =
                &mut self.stream_inputs[stream_input_index..(stream_input_index + node.stream_input_count)];

            // process the node
            if midi_inputs.iter().any(|midi_input| midi_input.is_some()) {
                node.node.accept_midi_inputs(midi_inputs);
            }

            if value_inputs.iter().any(|value_input| value_input.is_some()) {
                node.node.accept_value_inputs(value_inputs);
            }

            let res = node.node.process(
                NodeProcessState {
                    current_time,
                    script_engine,
                    global_state,
                },
                stream_inputs,
                &mut stream_staging[0..node.stream_output_count],
            );

            match res {
                Ok(res_warnings) => warnings.append_warnings(res_warnings.warnings),
                Err(err) => errors.push((node.node_index, err)),
            }

            node.node.get_midi_outputs(&mut midi_staging[0..node.midi_output_count]);
            node.node
                .get_value_outputs(&mut value_staging[0..node.value_output_count]);

            // now, send the outputs to where they need to go
            for _ in 0..node.stream_output_mappings {
                let mapping = self.stream_output_mappings[stream_output_mappings_i];
                self.stream_inputs[mapping.1] = stream_staging[mapping.0];

                stream_output_mappings_i += 1;
            }

            for _ in 0..node.midi_output_mappings {
                let mapping = self.midi_output_mappings[midi_output_mappings_i];
                self.midi_inputs[mapping.1] = midi_staging[mapping.0].clone();

                midi_output_mappings_i += 1;
            }

            for _ in 0..node.value_output_mappings {
                let mapping = self.value_output_mappings[value_output_mappings_i];
                self.value_inputs[mapping.1] = value_staging[mapping.0].clone();

                value_output_mappings_i += 1;
            }

            // Reset staging
            for i in 0..node.stream_output_count {
                stream_staging[i] = 0.0;
            }

            for i in 0..node.midi_output_count {
                midi_staging[i] = None;
            }

            for i in 0..node.value_output_count {
                value_staging[i] = None;
            }
        }

        if self.reset_needed {
            self.reset_default_inputs();
            self.reset_needed = false;
        }

        Ok(())
    }

    fn reset_default_inputs(&mut self) {
        for midi_index in &self.midi_default_indexes {
            self.midi_inputs[*midi_index] = None;
        }

        for value_index in &self.value_default_indexes {
            self.value_inputs[*value_index] = None;
        }
    }

    pub fn get_node_mut(&mut self, index: NodeIndex) -> Option<&mut NodeVariant> {
        self.nodes
            .iter_mut()
            .find(|node| node.node_index == index)
            .map(|node_state| &mut node_state.node)
    }
}
