use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct MixerNode {}

impl NodeRuntime for MixerNode {
    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        _midi_store: &mut OscStore,
        _resources: &[Resource],
    ) {
        for mut stream_out in outs.streams() {
            for channel_out in stream_out.iter_mut() {
                channel_out.fill(0.0);
            }
        }

        for stream_in in ins.streams() {
            for (channel_in, channel_out) in stream_in.iter().zip(outs.stream(0).iter_mut()) {
                for (frame_in, frame_out) in channel_in.iter().zip(channel_out.iter_mut()) {
                    *frame_out += *frame_in;
                }
            }
        }
    }
}

impl Node for MixerNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        MixerNode {}
    }

    fn get_io(context: NodeGetIoContext, props: SeaHashMap<String, Property>) -> NodeIo {
        let polyphony = default_channels(&props, context.default_channel_count);

        let mut node_rows = vec![
            with_channels(context.default_channel_count),
            stream_output("audio", polyphony),
        ];
        let input_count = context.connected_inputs.len() + 1;

        for i in 0..input_count {
            node_rows.push(NodeRow::Input(
                Socket::WithData(
                    "input_numbered".into(),
                    (i + 1).to_string(),
                    SocketType::Stream,
                    polyphony,
                ),
                SocketValue::Stream(0.0),
            ));
        }

        NodeIo::simple(node_rows)
    }
}
