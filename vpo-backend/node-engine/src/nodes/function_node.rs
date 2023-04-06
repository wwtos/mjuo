use crate::nodes::prelude::*;
use crate::traversal::traverser::Traverser;

#[derive(Debug, Clone)]
pub struct FunctionNode {
    traverser: Traverser,
    child_io_nodes: Option<(NodeIndex, NodeIndex)>,
}

impl Default for FunctionNode {
    fn default() -> FunctionNode {
        FunctionNode {
            traverser: Traverser::default(),
            child_io_nodes: None,
        }
    }
}

impl NodeRuntime for FunctionNode {
    fn init(&mut self, state: NodeInitState, child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        if let Some(graph_and_io) = child_graph {
            let NodeGraphAndIo {
                graph,
                input_index,
                output_index,
            } = graph_and_io;

            self.traverser = Traverser::get_traverser(
                graph_and_io.graph,
                state.graph_manager,
                state.script_engine,
                state.global_state,
                state.current_time,
            )?;
            self.child_io_nodes = Some((input_index, output_index));
        }

        InitResult::nothing()
    }

    fn process(
        &mut self,
        _state: NodeProcessState,
        _streams_in: &[f32],
        _streams_out: &mut [f32],
    ) -> Result<NodeOk<()>, NodeError> {
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
    fn get_io(props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
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
