use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct UpDownMixerNode {}

impl NodeRuntime for UpDownMixerNode {
    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        _midi_store: &mut MidiStore,
        _resources: &[Resource],
    ) {
        if ins.streams_len() <= outs.streams_len() {
            for (i, frame_out) in outs.stream(0).iter_mut().enumerate() {
                // rotate through the inputs to fill the outputs
                let frame_in = &ins.stream(0)[i % ins.stream(0).len()];

                frame_out.copy_from_slice(frame_in);
            }
        } else {
            // clear the last output
            for frame_out in outs.stream(0).iter_mut() {
                frame_out.fill(0.0);
            }

            for (i, frame_in) in ins.stream(0).iter().enumerate() {
                // rotate through the outputs

                let channel_i = i % outs.stream(0).len();
                let frame_out = &mut outs.stream(0)[channel_i];

                for (sample_in, sample_out) in frame_in.iter().zip(frame_out.iter_mut()) {
                    *sample_out += *sample_in;
                }
            }
        }
    }
}

impl Node for UpDownMixerNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        UpDownMixerNode {}
    }

    fn get_io(context: NodeGetIoContext, props: SeaHashMap<String, Property>) -> NodeIo {
        let channels = default_channels(&props, context.default_channel_count);

        let input_channels = props
            .get("input_channels")
            .and_then(|x| x.as_integer().map(|n| n.max(1) as usize))
            .unwrap_or(channels);
        let output_channels = props
            .get("output_channels")
            .and_then(|x| x.as_integer().map(|n| n.max(1) as usize))
            .unwrap_or(channels);

        NodeIo::simple(vec![
            property(
                "input_channels",
                PropertyType::Integer,
                Property::Integer(channels as i32),
            ),
            property(
                "output_channels",
                PropertyType::Integer,
                Property::Integer(channels as i32),
            ),
            stream_input("audio", input_channels),
            stream_output("audio", output_channels),
        ])
    }
}
