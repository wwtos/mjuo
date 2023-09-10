use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct GainNode {
    gain: f32,
}

impl NodeRuntime for GainNode {
    fn process(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins,
        outs: Outs,
        resources: &[&dyn Any],
    ) -> NodeResult<()> {
        if ins.values[0][0].is_some() {
            self.gain = ins.values[0][0].as_float().unwrap_or(0.0);
        }

        for (streams_in, streams_out) in ins.streams[0].iter().zip(outs.streams[0].iter_mut()) {
            for (frame_in, frame_out) in streams_in.iter().zip(streams_out.iter_mut()) {
                *frame_out = *frame_in * self.gain;
            }
        }

        NodeOk::no_warnings(())
    }
}

impl Node for GainNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        GainNode { gain: 0.0 }
    }

    fn get_io(context: NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
        let polyphony = default_channels(&props, context.default_channel_count);

        NodeIo::simple(vec![
            with_channels(context.default_channel_count),
            stream_input("audio", polyphony),
            value_input("gain", Primitive::Float(0.0), 1),
            stream_output("audio", polyphony),
        ])
    }
}
