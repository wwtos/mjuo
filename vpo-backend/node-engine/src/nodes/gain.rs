use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct GainNode {
    gain: f32,
}

impl NodeRuntime for GainNode {
    fn init(&mut self, state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        if let Some(Property::Float(gain)) = state.props.get("default_gain") {
            self.gain = gain.clamp(0.0, 1.0);
        }

        InitResult::nothing()
    }

    fn process(
        &mut self,
        _state: NodeProcessState,
        streams_in: &[&[f32]],
        streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        for i in 0..streams_in[0].len() {
            streams_out[0][i] = streams_in[0][i] * streams_in[1][i];
        }

        NodeOk::no_warnings(())
    }
}

impl Node for GainNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        GainNode { gain: 0.2 }
    }

    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            stream_input(register("audio"), 0.0),
            stream_input(register("gain"), 0.0),
            stream_output(register("audio"), 0.0),
        ])
    }
}
