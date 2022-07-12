use std::collections::HashMap;

use rhai::Engine;

use crate::connection::StreamSocketType;
use crate::node::{Node, InitResult, NodeRow};
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
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0)
        ])
    }
}