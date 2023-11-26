use crate::nodes::prelude::*;

#[derive(Debug, Default, Clone)]
pub struct DummyNode;

impl NodeRuntime for DummyNode {}

impl Node for DummyNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        DummyNode
    }

    fn get_io(_context: &NodeGetIoContext, _props: HashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![])
    }
}
