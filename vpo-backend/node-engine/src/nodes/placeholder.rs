use crate::nodes::prelude::*;

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
impl NodeRuntime for Placeholder {}

impl Node for Placeholder {
    fn get_io(props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        unreachable!("Placeholder never replaced after deserialization")
    }
}
