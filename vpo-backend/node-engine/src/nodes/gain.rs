use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct GainNode {
    gain: f32,
}

impl NodeRuntime for GainNode {
    fn process<'a, 'arena: 'a, 'brand>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a, 'arena, 'brand>,
        outs: Outs<'a, 'arena, 'brand>,
        _token: &mut GhostToken<'brand>,
        _arena: &'arena BuddyArena,
        _resources: &[&Resource],
    ) -> NodeResult<()> {
        if ins.values[0][0].get().is_some() {
            self.gain = ins.values[0][0].get().as_float().unwrap_or(0.0);
        }

        for (frame_in, frame_out) in ins.streams[0].iter().zip(outs.streams[0].iter()) {
            for (sample_in, sample_out) in frame_in.iter().zip(frame_out.iter()) {
                sample_out.set(sample_in.get() * self.gain);
            }
        }

        NodeOk::no_warnings(())
    }
}

impl Node for GainNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        GainNode { gain: 0.0 }
    }

    fn get_io(context: &NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
        let polyphony = default_channels(&props, context.default_channel_count);

        NodeIo::simple(vec![
            with_channels(context.default_channel_count),
            stream_input("audio", polyphony),
            value_input("gain", Primitive::Float(0.0), 1),
            stream_output("audio", polyphony),
        ])
    }
}
