use arr_macro::arr;
use ddgg::Edge;
use rhai::Engine;

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
    stream_inputs: usize,
    stream_outputs: usize,
    stream_output_mappings: usize,
    midi_inputs: usize,
    midi_outputs: usize,
    midi_output_mappings: usize,
    value_inputs: usize,
    value_outputs: usize,
    value_output_mappings: usize,
}

#[derive(Debug, Clone)]
pub struct Traverser {
    nodes: Vec<NodeState>,
    stream_inputs: Vec<f32>,
    stream_output_mappings: Vec<(usize, usize)>,
    midi_inputs: Vec<Option<MidiBundle>>,
    midi_output_mappings: Vec<(usize, usize)>,
    value_inputs: Vec<Option<Primitive>>,
    value_output_mappings: Vec<(usize, usize)>,
}

impl Default for Traverser {
    fn default() -> Self {
        Self::new()
    }
}

impl Traverser {
    pub fn new() -> Self {
        Traverser {
            nodes: vec![],
            stream_inputs: vec![],
            stream_output_mappings: vec![],
            midi_inputs: vec![],
            midi_output_mappings: vec![],
            value_inputs: vec![],
            value_output_mappings: vec![],
        }
    }

    pub fn get_traverser(
        graph_index: GraphIndex,
        graph_manager: &GraphManager,
        script_engine: &Engine,
        global_state: &GlobalState,
        current_time: i64,
    ) -> Result<Traverser, NodeError> {
        let mut traverser = Traverser::new();

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

        self.nodes.clear();
        self.stream_inputs.clear();
        self.stream_output_mappings.clear();
        self.midi_inputs.clear();
        self.midi_output_mappings.clear();
        self.value_inputs.clear();
        self.value_output_mappings.clear();

        let mut errors: Vec<(NodeIndex, NodeError)> = vec![];
        let mut warnings: WarningBuilder = WarningBuilder::new();

        for index in &traversal_order {
            // create and init the node
            let node_wrapper = graph.get_node(*index)?;

            let mut variant = new_variant(&node_wrapper.get_node_type(), &global_state.sound_config)?;

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

            let init_result = match init_result_res {
                Ok(init_result) => init_result,
                Err(err) => {
                    errors.push((*index, err));
                    continue;
                }
            };

            warnings.append_warnings(init_result.warnings);

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
            self.nodes.push(NodeState {
                node: variant,
                node_index: *index,
                stream_index: self.stream_inputs.len(),
                midi_index: self.midi_inputs.len(),
                value_index: self.value_inputs.len(),
                linked_to_ui,
                stream_inputs: stream_input_defaults.len(),
                stream_outputs: stream_outputs,
                stream_output_mappings: 0,
                midi_inputs: midi_input_defaults.len(),
                midi_outputs: midi_outputs,
                midi_output_mappings: 0,
                value_inputs: midi_input_defaults.len(),
                value_outputs: value_outputs,
                value_output_mappings: 0,
            });

            // populate the input lists with the defaults (which will be read every time)
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

                        for input in to.list_input_sockets() {
                            if input == socket {
                                let to_local_node = self.nodes.iter().find(|x| x.node_index == to_index).unwrap();

                                match socket.socket_type() {
                                    SocketType::Stream => self
                                        .stream_output_mappings
                                        .push((stream_index, to_local_node.stream_index + other_stream_index)),
                                    SocketType::Midi => self
                                        .midi_output_mappings
                                        .push((midi_index, other_midi_index + to_local_node.midi_index)),
                                    SocketType::Value => self
                                        .value_output_mappings
                                        .push((value_index, other_value_index + to_local_node.value_index)),
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

        println!("{:#?}", self);

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
            let midi_inputs = &self.midi_inputs[midi_input_index..(midi_input_index + node.midi_inputs)];
            let value_inputs = &self.value_inputs[value_input_index..(value_input_index + node.value_inputs)];
            let stream_inputs = &mut self.stream_inputs[stream_input_index..(stream_input_index + node.stream_inputs)];

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
                &mut stream_staging[0..node.stream_outputs],
            );

            match res {
                Ok(res_warnings) => warnings.append_warnings(res_warnings.warnings),
                Err(err) => errors.push((node.node_index, err)),
            }

            node.node.get_midi_outputs(&mut midi_staging[0..node.midi_outputs]);
            node.node.get_value_outputs(&mut value_staging[0..node.value_outputs]);

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
            for i in 0..node.stream_outputs {
                stream_staging[i] = 0.0;
            }

            for i in 0..node.midi_outputs {
                midi_staging[i] = None;
            }

            for i in 0..node.value_outputs {
                value_staging[i] = None;
            }
        }

        Ok(())
    }

    pub fn get_node_mut(&mut self, index: NodeIndex) -> Option<&mut NodeVariant> {
        self.nodes
            .iter_mut()
            .find(|node| node.node_index == index)
            .map(|node_state| &mut node_state.node)
    }
}
