use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct GainNode {
    gain: f32,
}

impl NodeRuntime for GainNode {
    fn process(
        &mut self,
        _globals: NodeProcessGlobals,
        ins: Ins,
        outs: Outs,
        _resources: &[Option<(ResourceIndex, &dyn Any)>],
    ) -> NodeResult<()> {
        if let Some(gain) = ins.values[0] {
            self.gain = gain.as_float().unwrap_or(0.0);
        }

        for (frame_in, frame_out) in ins.streams[0].iter().zip(outs.streams[0].iter_mut()) {
            *frame_out = *frame_in * self.gain;
        }

        NodeOk::no_warnings(())
    }
}

impl Node for GainNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        GainNode { gain: 0.0 }
    }

    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            stream_input(register("audio")),
            value_input(register("gain"), Primitive::Float(0.0)),
            stream_output(register("audio")),
        ])
    }
}
