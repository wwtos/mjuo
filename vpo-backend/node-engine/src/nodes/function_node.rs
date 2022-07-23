use std::collections::HashMap;

use rhai::Engine;

use crate::connection::{SocketDirection, SocketType, StreamSocketType};
use crate::node::{InitResult, Node, NodeIndex, NodeRow};
use crate::node_graph::NodeGraph;
use crate::{property::Property, socket_registry::SocketRegistry};

#[derive(Debug, Clone, Default)]
pub struct FunctionNode {}

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

    fn init_graph(&mut self, graph: &mut NodeGraph, input_node: &NodeIndex, output_node: &NodeIndex) {}
}
