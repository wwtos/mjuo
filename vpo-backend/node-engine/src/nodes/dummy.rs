use std::collections::HashMap;

use rhai::Engine;

use crate::{
    node::{InitResult, Node},
    property::Property,
    socket_registry::SocketRegistry,
};

#[derive(Debug, Default, Clone)]
pub struct DummyNode {}

impl Node for DummyNode {
    fn init(
        &mut self,
        _props: &HashMap<String, Property>,
        _registry: &mut SocketRegistry,
        _scripting_engine: &Engine,
    ) -> InitResult {
        InitResult::simple(vec![])
    }
}
