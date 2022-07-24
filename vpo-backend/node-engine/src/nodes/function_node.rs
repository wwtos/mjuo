use std::collections::HashMap;

use rhai::Engine;

use crate::connection::{SocketDirection, SocketType, StreamSocketType};
use crate::errors::ErrorsAndWarnings;
use crate::node::{InitResult, Node, NodeIndex, NodeRow};
use crate::node_graph::NodeGraph;
use crate::traversal::traverser::Traverser;
use crate::{property::Property, socket_registry::SocketRegistry};

#[derive(Debug, Clone)]
pub struct FunctionNode {
    local_graph: NodeGraph,
    traverser: Traverser,
    input: f32,
    output: f32,
    is_first_time: bool,
    inner_input_node: NodeIndex,
    inner_output_node: NodeIndex
}

impl Default for FunctionNode {
    fn default() -> FunctionNode {
        FunctionNode {
            local_graph: NodeGraph::new(),
            traverser: Traverser::default(),
            input: 0_f32,
            output: 0_f32,
            is_first_time: true,
            inner_input_node: NodeIndex { index: 0, generation: 0 },
            inner_output_node: NodeIndex { index: 0, generation: 0 },
        }
    }
}

impl Node for FunctionNode {
    fn init(
        &mut self,
        _props: &HashMap<String, Property>,
        _registry: &mut SocketRegistry,
        _scripting_engine: &Engine,
    ) -> InitResult {
        InitResult::simple(vec![
            NodeRow::StreamInput(StreamSocketType::Audio, 0.0),
            NodeRow::InnerGraph,
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0),
        ])
    }

    fn get_inner_graph_socket_list(&self, _registry: &mut SocketRegistry) -> Vec<(SocketType, SocketDirection)> {
        vec![
            (SocketType::Stream(StreamSocketType::Audio), SocketDirection::Input),
            (SocketType::Stream(StreamSocketType::Audio), SocketDirection::Output),
        ]
    }

    fn accept_stream_input(&mut self, _socket_type: &StreamSocketType, value: f32) {
        self.input = value;
    }

    fn get_stream_output(&self, _socket_type: &StreamSocketType) -> f32 {
        self.output
    }

    fn init_graph(&mut self, graph: &mut NodeGraph, input_node: NodeIndex, output_node: NodeIndex) {
        self.local_graph = graph.clone();
        self.traverser = Traverser::get_traverser(&self.local_graph);
        self.is_first_time = true;
        self.inner_input_node = input_node;
        self.inner_output_node = output_node;
    }

    fn process(
        &mut self,
        current_time: i64,
        scripting_engine: &Engine,
        inner_graph: Option<(&mut NodeGraph, &Traverser)>,
    ) -> Result<(), ErrorsAndWarnings> {
        let subgraph_input_node = self.local_graph.get_node_mut(&self.inner_input_node).unwrap();
        subgraph_input_node.accept_stream_input(&StreamSocketType::Audio, self.input);

        self.traverser.traverse(&mut self.local_graph, self.is_first_time, current_time, scripting_engine)?;

        let subgraph_output_node = self.local_graph.get_node_mut(&self.inner_output_node).unwrap();
        self.output = subgraph_output_node.get_stream_output(&StreamSocketType::Audio);
        
        self.is_first_time = false;

        Ok(())
    }
}
