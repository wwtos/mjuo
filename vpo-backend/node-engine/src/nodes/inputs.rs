use std::iter::repeat_with;

use crate::nodes::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct InputsNode {
    midis: Option<MidiChannel>,
    streams: Vec<Vec<f32>>,
}

impl InputsNode {
    pub fn set_midis(&mut self, midis: MidiChannel) {
        self.midis = Some(midis);
    }

    pub fn streams_mut(&mut self) -> &mut Vec<Vec<f32>> {
        &mut self.streams
    }
}

impl NodeRuntime for InputsNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let channels = default_channels(&params.props, params.default_channel_count);

        let type_str = params.props.get("type").and_then(|x| x.clone().as_multiple_choice());
        let socket_type = match type_str.as_ref().map(|x| x.as_str()) {
            Some("stream") => SocketType::Stream,
            Some("midi") => SocketType::Midi,
            _ => SocketType::Stream,
        };

        match socket_type {
            SocketType::Stream => {
                self.midis = None;
                self.streams = repeat_with(|| vec![0.0; params.sound_config.buffer_size])
                    .take(channels)
                    .collect();
            }
            SocketType::Midi => {
                self.midis = None;
                self.streams = vec![];
            }
            _ => {}
        }

        InitResult::nothing()
    }

    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        _ins: Ins<'a>,
        mut outs: Outs<'a>,
        midi_store: &mut MidiStore,
        _resources: &[Resource],
    ) -> NodeResult<()> {
        if outs.midis_len() > 0 {
            if let Some(midis) = &mut self.midis {
                outs.midi(0)[0] = midi_store.add_midi(midis.drain(..));

                self.midis = None;
            } else {
                outs.midi(0)[0] = None;
            }
        }

        if outs.streams_len() > 0 {
            for (channel_out, channel) in outs.stream(0).iter_mut().zip(self.streams.iter()) {
                channel_out.copy_from_slice(&channel[..]);
            }
        }

        NodeOk::no_warnings(())
    }
}

impl Node for InputsNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        InputsNode {
            midis: None,
            streams: vec![],
        }
    }

    fn get_io(context: &NodeGetIoContext, props: SeaHashMap<String, Property>) -> NodeIo {
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
