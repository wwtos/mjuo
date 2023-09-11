use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct MixerNode {}

impl NodeRuntime for MixerNode {
    fn process<'brand>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'_, 'brand>,
        outs: Outs<'_, 'brand>,
        token: &mut GhostToken<'brand>,
        resources: &[&dyn Any],
    ) -> NodeResult<()> {
        for stream_out in outs.streams {
            for channel_out in stream_out.iter_mut() {
                for frame_out in channel_out.iter_mut() {
                    *frame_out.borrow_mut(token) = 0.0;
                }
            }
        }

        for stream_in in ins.streams {
            for (channel_in, channel_out) in stream_in.iter().zip(outs.streams[0].iter_mut()) {
                for (frame_in, frame_out) in channel_in.iter().zip(channel_out.iter_mut()) {
                    *frame_out.borrow_mut(token) += *frame_in.borrow(token);
                }
            }
        }

        NodeOk::no_warnings(())
    }
}

impl Node for MixerNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        MixerNode {}
    }

    fn get_io(context: NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
        let polyphony = default_channels(&props, context.default_channel_count);

        let mut node_rows = vec![
            with_channels(context.default_channel_count),
            property("input_count", PropertyType::Integer, Property::Integer(2)),
            stream_output("audio", polyphony),
        ];

        let input_count = props
            .get("input_count")
            .and_then(|x| x.clone().as_integer())
            .unwrap_or(2);

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
