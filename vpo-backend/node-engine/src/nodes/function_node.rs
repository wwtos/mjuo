use crate::connection::{SocketDirection, SocketType, StreamSocketType};
use crate::errors::{NodeError, NodeOk, NodeResult};
use crate::node::{InitResult, Node, NodeGraphAndIo, NodeIndex, NodeInitState, NodeProcessState, NodeRow};
use crate::node_graph::NodeGraph;
use crate::traversal::traverser::Traverser;

#[derive(Debug, Clone)]
pub struct FunctionNode {
    local_graph: NodeGraph,
    traverser: Traverser,
    child_io_nodes: Option<(NodeIndex, NodeIndex)>,
}

impl Default for FunctionNode {
    fn default() -> FunctionNode {
        FunctionNode {
            local_graph: NodeGraph::new(),
            traverser: Traverser::default(),
            child_io_nodes: None,
        }
    }
}

impl Node for FunctionNode {
    fn init(&mut self, _state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        NodeOk::no_warnings(InitResult {
            did_rows_change: false,
            node_rows: vec![
                NodeRow::StreamInput(StreamSocketType::Audio, 0.0, false),
                NodeRow::InnerGraph,
                NodeRow::StreamOutput(StreamSocketType::Audio, 0.0, false),
            ],
            changed_properties: None,
            child_graph_io: Some(vec![
                (SocketType::Stream(StreamSocketType::Audio), SocketDirection::Input),
                (SocketType::Stream(StreamSocketType::Audio), SocketDirection::Output),
            ]),
        })
    }

    fn post_init(&mut self, init_state: NodeInitState, graph_and_io: Option<NodeGraphAndIo>) -> NodeResult<()> {
        if let Some(graph_and_io) = graph_and_io {
            let NodeGraphAndIo {
                graph,
                input_index,
                output_index,
            } = graph_and_io;

            self.local_graph = graph.clone();
            self.traverser = Traverser::get_traverser(&mut self.local_graph, init_state)?;
            self.child_io_nodes = Some((input_index, output_index));
        }

        NodeOk::no_warnings(())
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
