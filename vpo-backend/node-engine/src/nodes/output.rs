use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct OutputNode {
    values_received: Vec<f32>,
}

impl OutputNode {
    pub fn get_values_received(&self) -> Vec<f32> {
        self.values_received.clone()
    }
}

impl NodeRuntime for OutputNode {
    fn process(
        &mut self,
        _globals: NodeProcessGlobals,
        ins: Ins,
        _outs: Outs,
        _resources: &[Option<(ResourceIndex, &dyn Any)>],
    ) -> NodeResult<()> {
        self.values_received.resize(ins.streams[0].len(), 0.0);
        self.values_received.clone_from_slice(ins.streams[0]);

        NodeOk::no_warnings(())
    }
}

impl Node for OutputNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        OutputNode {
            values_received: vec![],
        }
    }

    fn get_io(_props: HashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![stream_input("audio")])
    }
}
