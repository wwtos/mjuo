use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct OutputNode {
    value_received: f32,
}

impl Default for OutputNode {
    fn default() -> Self {
        OutputNode { value_received: 0.0 }
    }
}

impl OutputNode {
    pub fn get_value_received(&self) -> f32 {
        self.value_received
    }
}

impl NodeRuntime for OutputNode {
    fn process(&mut self, state: NodeProcessState, streams_in: &[f32], streams_out: &mut [f32]) -> NodeResult<()> {
        self.value_received = streams_in[0];

        NodeOk::no_warnings(())
    }
}

impl Node for OutputNode {
    fn get_io(props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![stream_input(register("audio"), 0.0)])
    }
}
