use std::iter::repeat_with;

use crate::nodes::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct OutputsNode {
    midis: Option<MidiChannel>,
    midi_stale: bool,
    streams: Vec<Vec<f32>>,
}

impl OutputsNode {
    pub fn get_streams(&self) -> &Vec<Vec<f32>> {
        &self.streams
    }

    pub fn get_midis(&self) -> &Option<MidiChannel> {
        &self.midis
    }
}

impl NodeRuntime for OutputsNode {
    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        _outs: Outs<'a>,
        midi_store: &mut MidiStore,
        _resources: &[Resource],
    ) {
        if self.midi_stale {
            self.midis = None;
        }

        if ins.midis_len() > 0 {
            if let Some(midi_index) = &ins.midi(0)[0] {
                self.midis = midi_store.borrow_midi(midi_index).map(|midi| midi.to_vec());
                self.midi_stale = false;
            } else {
                self.midi_stale = true;
            }
        }

        if ins.streams_len() > 0 {
            for (channel_in, local_channel_in) in ins.stream(0).iter().zip(self.streams.iter_mut()) {
                local_channel_in.clear();

                local_channel_in.extend(channel_in.iter());
            }
        }
    }

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
}

impl Node for OutputsNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        OutputsNode {
            midis: None,
            midi_stale: true,
            streams: vec![],
        }
    }

    fn get_io(context: NodeGetIoContext, props: SeaHashMap<String, Property>) -> NodeIo {
        let channels = default_channels(&props, context.default_channel_count);

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
                node_rows.push(stream_input("audio", channels));
            }
            SocketType::Midi => {
                node_rows.push(midi_input("midi", 1));
            }
            _ => {}
        }

        NodeIo::simple(node_rows)
    }
}
