use std::iter::repeat_with;

use common::osc::{BundleWriter, OscTime};

use crate::nodes::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct InputsNode {
    osc: Vec<u8>,
    streams: Vec<Vec<f32>>,
}

impl InputsNode {
    pub fn osc_ref(&mut self) -> &Vec<u8> {
        &self.osc
    }

    pub fn osc_for_writing(&mut self) -> &mut Vec<u8> {
        if self.osc.is_empty() {
            // use bundle writer to write the header
            BundleWriter::start(Some(&mut self.osc), OscTime::default())
                .expect("enough space to write osc bundle header");
        }

        &mut self.osc
    }

    pub fn streams_mut(&mut self) -> &mut Vec<Vec<f32>> {
        &mut self.streams
    }
}

impl NodeRuntime for InputsNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let channels = params.get_channel_count();

        let type_str = params.props.get_multiple_choice("type")?;
        let socket_type = match type_str.as_str() {
            "stream" => SocketType::Stream,
            "midi" => SocketType::Midi,
            _ => SocketType::Stream,
        };

        match socket_type {
            SocketType::Stream => {
                self.osc.clear();
                self.streams = repeat_with(|| vec![0.0; params.sound_config.buffer_size])
                    .take(channels)
                    .collect();
            }
            SocketType::Midi => {
                self.osc.clear();
                self.streams = vec![];
            }
            _ => {}
        }

        InitResult::nothing()
    }

    fn process<'a>(
        &mut self,
        context: NodeProcessContext,
        _ins: Ins<'a>,
        mut outs: Outs<'a>,
        osc_store: &mut OscStore,
        _resources: &[Resource],
    ) {
        // do we have an osc output?
        if outs.oscs_len() > 0 {
            if !self.osc.is_empty() {
                outs.osc(0)[0] = osc_store.copy_from(&self.osc);
                self.osc.clear();
            } else {
                outs.osc(0)[0] = None;
            }
        }

        if outs.streams_len() > 0 {
            for (channel_out, channel) in outs.stream(0).iter_mut().zip(self.streams.iter()) {
                channel_out.copy_from_slice(&channel[..]);
            }
        }
    }
}

impl Node for InputsNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        InputsNode {
            osc: Vec::with_capacity(64),
            streams: vec![],
        }
    }

    fn get_io(context: NodeGetIoContext, props: SeaHashMap<String, Property>) -> NodeIo {
        let channels = default_channels(&props, context.default_channel_count);

        // NOTE: if you change any of the IO definitions here, MAKE SURE to update it in
        // nodes/polyphonic.rs
        let type_str = props.get("type").and_then(|x| x.clone().as_multiple_choice());
        let socket_type = match type_str.as_ref().map(|x| x.as_str()) {
            Some("stream") => SocketType::Stream,
            Some("midi") => SocketType::Midi,
            _ => SocketType::Stream,
        };

        let mut node_rows = vec![
            property("name", PropertyType::String, Property::String("".into())),
            multiple_choice("type", &["midi", "stream"], "stream"),
        ];

        match socket_type {
            SocketType::Stream => {
                node_rows.push(with_channels(context.default_channel_count));
                node_rows.push(stream_output("audio", channels));
            }
            SocketType::Midi => {
                node_rows.push(midi_output("midi", 1));
            }
            _ => {}
        }

        NodeIo::simple(node_rows)
    }
}
