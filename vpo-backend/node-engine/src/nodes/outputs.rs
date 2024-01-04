use std::iter::repeat;

use crate::nodes::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct OutputsNode {
    values: Vec<Vec<Primitive>>,
    midis: Vec<Vec<MidiChannel>>,
    streams: Vec<Vec<Vec<f32>>>,
}

impl OutputsNode {
    pub fn get_streams(&self) -> &Vec<Vec<Vec<f32>>> {
        &self.streams
    }

    pub fn get_midis(&self) -> &Vec<Vec<MidiChannel>> {
        &self.midis
    }

    pub fn get_values(&self) -> &Vec<Vec<Primitive>> {
        &self.values
    }
}

impl NodeRuntime for OutputsNode {
    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        _outs: Outs<'a>,
        midi_store: &mut MidiStoreInterface,
        _resources: &[Resource],
    ) -> NodeResult<()> {
        for (socket_in, socket) in ins.midis().zip(self.midis.iter_mut()) {
            for (channel_in, channel) in socket_in.iter().zip(socket.iter_mut()) {
                channel.clear();

                if let Some(midi) = channel_in {
                    channel.clone_from_slice(midi_store.borrow_midi(midi).unwrap());
                }
            }
        }

        for (socket_in, local_in) in ins.values().zip(self.values.iter_mut()) {
            local_in.clear();
            local_in.extend(socket_in.iter());
        }

        for (socket_in, local_in) in ins.streams().zip(self.streams.iter_mut()) {
            for (channel_in, local_channel_in) in socket_in.iter().zip(local_in.iter_mut()) {
                local_channel_in.clear();

                local_channel_in.extend(channel_in.iter());
            }
        }

        NodeOk::no_warnings(())
    }

    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let buffer_size = params.sound_config.buffer_size;

        if let Some(Property::SocketList(sockets)) = params.props.get("socket_list") {
            self.midis.clear();
            self.values.clear();
            self.streams.clear();

            for socket in sockets {
                let socket_type = socket.socket_type();
                let channels = socket.channels();

                match socket_type {
                    SocketType::Stream => {
                        self.streams
                            .push(repeat(vec![0.0; buffer_size]).take(channels).collect());
                    }
                    SocketType::Midi => {
                        self.midis.push(repeat(vec![]).take(channels).collect());
                    }
                    SocketType::Value => {
                        self.values.push(repeat(Primitive::None).take(channels).collect());
                    }
                    SocketType::NodeRef => {}
                }
            }

            let midi_outputs = sockets
                .iter()
                .filter(|output| output.socket_type() == SocketType::Midi)
                .count();

            let value_outputs = sockets
                .iter()
                .filter(|output| output.socket_type() == SocketType::Value)
                .count();

            self.midis.resize_with(midi_outputs, || vec![vec![]]);
            self.values.resize_with(value_outputs, || vec![Primitive::None]);
        }

        InitResult::nothing()
    }
}

impl Node for OutputsNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        OutputsNode {
            values: vec![],
            midis: vec![],
            streams: vec![],
        }
    }

    fn get_io(context: &NodeGetIoContext, props: SeaHashMap<String, Property>) -> NodeIo {
        let channels = default_channels(&props, context.default_channel_count);

        let mut node_rows = vec![
            with_channels(context.default_channel_count),
            property("name", PropertyType::String, Property::String("".into())),
            property("audio_count", PropertyType::Integer, Property::Integer(1)),
            property("midi_count", PropertyType::Integer, Property::Integer(1)),
        ];

        let streams = props
            .get("audio_count")
            .and_then(|x| x.as_integer())
            .unwrap_or(1)
            .max(1) as usize;
        let midis = props.get("midi_count").and_then(|x| x.as_integer()).unwrap_or(1).max(1) as usize;

        for i in 0..streams {
            node_rows.push(NodeRow::Input(
                Socket::WithData(
                    "audio_out_numbered".into(),
                    (i + 1).to_string(),
                    SocketType::Stream,
                    channels,
                ),
                SocketValue::None,
            ));
        }

        for i in 0..midis {
            node_rows.push(NodeRow::Input(
                Socket::WithData("midi_out_numbered".into(), (i + 1).to_string(), SocketType::Midi, 1),
                SocketValue::None,
            ));
        }

        NodeIo::simple(node_rows)
    }
}
