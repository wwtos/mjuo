use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct MixerNode {}

impl NodeRuntime for MixerNode {
    fn process(
        &mut self,
        globals: NodeProcessGlobals,
        ins: Ins,
        outs: Outs,
        resources: &[Option<(ResourceIndex, &dyn Any)>],
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

    fn get_io(props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        let mut node_rows = vec![
            NodeRow::Property("input_count".to_string(), PropertyType::Integer, Property::Integer(2)),
            stream_output(register("audio")),
        ];

        let input_count = props
            .get("input_count")
            .and_then(|x| x.clone().as_integer())
            .unwrap_or(2);

        for i in 0..input_count {
            node_rows.push(NodeRow::Input(
                Socket::Numbered(register("input-numbered"), i + 1, SocketType::Stream, 1),
                SocketValue::Stream(0.0),
            ));
        }

        NodeIo::simple(node_rows)
    }
}
