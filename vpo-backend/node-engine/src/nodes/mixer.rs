use std::borrow::Cow;

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct MixerNode {}

impl NodeRuntime for MixerNode {
    fn process(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins,
        outs: Outs,
        _resources: &[Option<(ResourceIndex, &dyn Any)>],
    ) -> NodeResult<()> {
        for (i, frame) in outs.streams[0].iter_mut().enumerate() {
            *frame = 0.0;

            for stream_in in ins.streams {
                *frame += stream_in[i];
            }
        }

        NodeOk::no_warnings(())
    }
}

impl Node for MixerNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        MixerNode {}
    }

    fn get_io(props: HashMap<String, Property>) -> NodeIo {
        let mut node_rows = vec![
            NodeRow::Property("input_count".to_string(), PropertyType::Integer, Property::Integer(2)),
            stream_output("audio"),
        ];

        let input_count = props
            .get("input_count")
            .and_then(|x| x.clone().as_integer())
            .unwrap_or(2);

        for i in 0..input_count {
            node_rows.push(NodeRow::Input(
                Socket::WithData(
                    Cow::Borrowed("input_numbered"),
                    (i + 1).to_string(),
                    SocketType::Stream,
                    1,
                ),
                SocketValue::Stream(0.0),
            ));
        }

        NodeIo::simple(node_rows)
    }
}
