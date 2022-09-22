use std::collections::HashMap;

use rhai::Engine;

use crate::{
    node::{InitResult, Node},
    property::Property,
    socket_registry::SocketRegistry,
};

#[derive(Debug, Default, Clone)]
pub struct Placeholder {
    node_type: String,
}

impl Placeholder {
    pub fn new(node_type: String) -> Placeholder {
        Placeholder { node_type }
    }
}

/// Placeholder
///
/// This holds the place during the deserialization process -- the code later
/// goes through and converts it into a proper node
impl Node for Placeholder {
    fn init(
        &mut self,
        _props: &HashMap<String, Property>,
        _registry: &mut SocketRegistry,
        _scripting_engine: &Engine,
    ) -> InitResult {
        unreachable!("placeholder node being initialized!")
    }
}
