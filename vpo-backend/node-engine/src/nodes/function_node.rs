use crate::connection::{SocketDirection, SocketType, StreamSocketType};
use crate::errors::{NodeError, NodeOk};
use crate::node::{InitResult, Node, NodeIndex, NodeInitState, NodeProcessState, NodeRow};
use crate::node_graph::NodeGraph;
use crate::socket_registry::SocketRegistry;
use crate::traversal::traverser::Traverser;

#[derive(Debug, Clone)]
pub struct FunctionNode {
    local_graph: NodeGraph,
    traverser: Traverser,
    input: f32,
    output: f32,
    is_first_time: bool,
    child_io_nodes: Option<(NodeIndex, NodeIndex)>,
}

impl Default for FunctionNode {
    fn default() -> FunctionNode {
        FunctionNode {
            local_graph: NodeGraph::new(),
            traverser: Traverser::default(),
            input: 0_f32,
            output: 0_f32,
            is_first_time: true,
            child_io_nodes: None,
        }
    }
}

impl Node for FunctionNode {
    fn init(&mut self, _state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        InitResult::simple(vec![
            NodeRow::StreamInput(StreamSocketType::Audio, 0.0, false),
            NodeRow::InnerGraph,
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0, false),
        ])
    }

    fn get_child_graph_socket_list(&self, _registry: &mut SocketRegistry) -> Vec<(SocketType, SocketDirection)> {
        vec![
            (SocketType::Stream(StreamSocketType::Audio), SocketDirection::Input),
            (SocketType::Stream(StreamSocketType::Audio), SocketDirection::Output),
        ]
    }

    fn accept_stream_input(&mut self, _socket_type: StreamSocketType, value: f32) {
        self.input = value;
    }

    fn get_stream_output(&self, _socket_type: StreamSocketType) -> f32 {
        self.output
    }

    fn init_graph(&mut self, graph: &mut NodeGraph, input_node: NodeIndex, output_node: NodeIndex) {
        self.local_graph = graph.clone();
        self.traverser = Traverser::get_traverser(&self.local_graph);
        self.is_first_time = true;
        self.child_io_nodes = Some((input_node, output_node));
    }

    fn process(&mut self, state: NodeProcessState) -> Result<NodeOk<()>, NodeError> {
        let (child_input_node, child_output_node) = self.child_io_nodes.unwrap();

        let subgraph_input_node = self.local_graph.get_node_mut(child_input_node).unwrap();
        subgraph_input_node.accept_stream_input(StreamSocketType::Audio, self.input);

        self.traverser
            .traverse(
                &mut self.local_graph,
                self.is_first_time,
                state.current_time,
                state.script_engine,
                state.global_state,
            )
            .map_err(|err| NodeError::InnerGraphErrors {
                errors_and_warnings: err,
            })?;

        let subgraph_output_node = self.local_graph.get_node_mut(child_output_node).unwrap();
        self.output = subgraph_output_node.get_stream_output(StreamSocketType::Audio);

        self.is_first_time = false;

        NodeOk::no_warnings(())
    }
}
