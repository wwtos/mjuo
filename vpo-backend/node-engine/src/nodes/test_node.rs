use crate::nodes::prelude::*;

#[derive(Debug, Default, Clone)]
pub struct TestNode;

impl NodeRuntime for TestNode {}

impl Node for TestNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        TestNode
    }

    fn get_io(_context: &NodeGetIoContext, _props: SeaHashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            stream_input("audio", 1),
            stream_input("gain", 1),
            midi_input("midi", 1),
            stream_output("audio", 1),
            value_output("gate", 1),
        ])
    }
}
