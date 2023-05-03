use crate::{nodes::prelude::*, traversal::buffered_traverser::BufferedTraverser};

#[derive(Debug, Clone)]
pub struct FunctionNode {
    traverser: BufferedTraverser,
    child_io_nodes: Option<(NodeIndex, NodeIndex)>,
}

impl NodeRuntime for FunctionNode {
    fn init(&mut self, state: NodeInitState, child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        let mut warning = None;

        if let Some(graph_and_io) = child_graph {
            let NodeGraphAndIo {
                graph: _,
                input_index,
                output_index,
            } = graph_and_io;

            let (traverser, errors_and_warnings) = BufferedTraverser::new(
                graph_and_io.graph,
                state.graph_manager,
                state.script_engine,
                state.resources,
                state.current_time,
                state.sound_config.clone(),
            )?;
            self.traverser = traverser;

            if errors_and_warnings.any() {
                warning = Some(NodeWarning::InternalErrorsAndWarnings { errors_and_warnings });
            }

            self.child_io_nodes = Some((input_index, output_index));
        }

        InitResult::warning(warning)
    }

    fn process(
        &mut self,
        _state: NodeProcessState,
        _streams_in: &[&[f32]],
        _streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        // let (child_input_node, child_output_node) = self.child_io_nodes.unwrap();

        // let subgraph_input_node = self.local_graph.get_node_mut(child_input_node).unwrap();
        // subgraph_input_node.accept_stream_input(StreamSocketType::Audio, streams_in[0]);

        // self.traverser
        //     .traverse(
        //         &mut self.local_graph,
        //         self.is_first_time,
        //         state.current_time,
        //         state.script_engine,
        //         state.global_state,
        //     )
        //     .map_err(|err| NodeError::InnerGraphErrors {
        //         errors_and_warnings: err,
        //     })?;

        // let subgraph_output_node = self.local_graph.get_node_mut(child_output_node).unwrap();
        // streams_out[0] = subgraph_output_node.get_stream_output(StreamSocketType::Audio);

        // self.is_first_time = false;

        NodeOk::no_warnings(())
    }
}

impl Node for FunctionNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        FunctionNode {
            traverser: BufferedTraverser::default(),
            child_io_nodes: None,
        }
    }

    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo {
            node_rows: vec![
                stream_input(register("audio"), 0.0),
                NodeRow::InnerGraph,
                stream_output(register("audio"), 0.0),
            ],
            child_graph_io: Some(vec![
                (
                    Socket::Simple(register("audio"), SocketType::Stream, 1),
                    SocketDirection::Input,
                ),
                (
                    Socket::Simple(register("audio"), SocketType::Stream, 1),
                    SocketDirection::Output,
                ),
            ]),
        }
    }
}
