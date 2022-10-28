use crate::errors::{NodeError, NodeOk};
use crate::node::{InitResult, Node, NodeInitState};

#[derive(Debug, Default, Clone)]
pub struct Placeholder {
    node_type: String,
}

impl Placeholder {
    pub fn new(node_type: String) -> Placeholder {
        Placeholder { node_type }
    }

    pub fn get_variant(&self) -> String {
        self.node_type.clone()
    }
}

/// Placeholder
///
/// This holds the place during the deserialization process -- the code later
/// goes through and converts it into a proper node
impl Node for Placeholder {
    fn init(&mut self, _state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        unreachable!("placeholder node being initialized!")
    }
}
