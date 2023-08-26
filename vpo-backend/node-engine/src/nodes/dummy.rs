use crate::nodes::prelude::*;

#[derive(Debug, Default, Clone)]
pub struct DummyNode;

impl NodeRuntime for DummyNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        InitResult::nothing()
    }

    fn process(
        &mut self,
        globals: NodeProcessGlobals,
        ins: Ins,
        outs: Outs,
        resources: &[Option<(ResourceIndex, &dyn Any)>],
    ) -> NodeResult<()> {
        ProcessResult::nothing()
    }
}

impl Node for DummyNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        DummyNode
    }

    fn get_io(_props: HashMap<String, Property>, _register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![])
    }
}
