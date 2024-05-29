use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct GainNode {
    gain: f32,
}

impl NodeRuntime for GainNode {
    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        _osc_store: &mut OscStore,
        _resources: &[Resource],
    ) {
        ins.value(0)[0].as_float().map(|gain| self.gain = gain);

        for (frame_in, frame_out) in ins.stream(0).iter().zip(outs.stream(0).iter_mut()) {
            for (sample_in, sample_out) in frame_in.iter().zip(frame_out.iter_mut()) {
                *sample_out = *sample_in * self.gain;
            }
        }
    }
}

impl Node for GainNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        GainNode { gain: 0.0 }
    }

    fn get_io(context: NodeGetIoContext, props: SeaHashMap<String, Property>) -> NodeIo {
        let polyphony = default_channels(&props, context.default_channel_count);

        NodeIo::simple(vec![
            with_channels(context.default_channel_count),
            stream_input("audio", polyphony),
            value_input("gain", Primitive::Float(0.0), 1),
            stream_output("audio", polyphony),
        ])
    }
}
