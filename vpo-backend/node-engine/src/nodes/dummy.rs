use crate::nodes::prelude::*;

#[derive(Debug, Default, Clone)]
pub struct DummyNode {}

impl NodeRuntime for DummyNode {}

impl Node for DummyNode {
    fn get_io(props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![])
    }
}
