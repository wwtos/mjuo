use std::iter::repeat_with;

use crate::nodes::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct OutputsNode {
    oscs: Option<Vec<u8>>,
    osc_stale: bool,
    streams: Vec<Vec<f32>>,
}

impl OutputsNode {
    pub fn get_streams(&self) -> &Vec<Vec<f32>> {
        &self.streams
    }

    pub fn get_oscs(&self) -> Option<&Vec<u8>> {
        self.oscs.as_ref()
    }
}

impl NodeRuntime for OutputsNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let channels = params.get_channel_count();

        let socket_type = match params.props.get_multiple_choice("type")?.as_str() {
            "stream" => SocketType::Stream,
            "osc" => SocketType::Osc,
            _ => SocketType::Stream,
        };

        match socket_type {
            SocketType::Stream => {
                self.oscs = None;
                self.streams = repeat_with(|| vec![0.0; params.sound_config.buffer_size])
                    .take(channels)
                    .collect();
            }
            SocketType::Osc => {
                self.oscs = None;
                self.streams = vec![];
            }
            _ => {}
        }

        InitResult::nothing()
    }

    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        _outs: Outs<'a>,
        osc_store: &mut OscStore,
        _resources: &[Resource],
    ) {
        if self.osc_stale {
            self.oscs = None;
        }

        if ins.oscs_len() > 0 {
            if let Some(osc_bytes) = &ins.osc(0)[0].get_messages(osc_store) {
                self.oscs = Some(osc_bytes.to_vec());
                self.osc_stale = false;
            } else {
                self.osc_stale = true;
            }
        }

        if ins.streams_len() > 0 {
            for (channel_in, local_channel_in) in ins.stream(0).iter().zip(self.streams.iter_mut()) {
                local_channel_in.clear();

                local_channel_in.extend(channel_in.iter());
            }
        }
    }
}

impl Node for OutputsNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        OutputsNode {
            oscs: None,
            osc_stale: true,
            streams: vec![],
        }
    }

    fn get_io(context: NodeGetIoContext, props: SeaHashMap<String, Property>) -> NodeIo {
        let channels = default_channels(&props, context.default_channel_count);

        let type_str = props.get("type").and_then(|x| x.clone().as_multiple_choice());
        let socket_type = match type_str.as_ref().map(|x| x.as_str()) {
            Some("stream") => SocketType::Stream,
            Some("osc") => SocketType::Osc,
            _ => SocketType::Stream,
        };

        let mut node_rows = vec![
            property("name", PropertyType::String, Property::String("".into())),
            multiple_choice("type", &["osc", "stream"], "stream"),
        ];

        match socket_type {
            SocketType::Stream => {
                node_rows.push(with_channels(context.default_channel_count));
                node_rows.push(stream_input("audio", channels));
            }
            SocketType::Osc => {
                node_rows.push(osc_input("osc", 1));
            }
            _ => {}
        }

        NodeIo::simple(node_rows)
    }
}
