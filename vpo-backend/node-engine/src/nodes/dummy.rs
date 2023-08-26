use crate::nodes::prelude::*;

#[derive(Debug, Default, Clone)]
pub struct DummyNode;

impl NodeRuntime for DummyNode {}

impl Node for DummyNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        DummyNode
    }

    fn get_io(_props: HashMap<String, Property>, _register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![])
    }
}
