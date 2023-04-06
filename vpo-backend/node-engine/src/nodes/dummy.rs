use crate::nodes::prelude::*;

#[derive(Debug, Default, Clone)]
pub struct DummyNode {}

impl NodeRuntime for DummyNode {}

impl Node for DummyNode {
    fn get_io(props: HashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![])
    }
}
