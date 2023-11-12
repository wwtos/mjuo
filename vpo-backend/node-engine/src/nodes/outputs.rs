use std::iter::repeat;

use crate::nodes::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct OutputsNode {
    values: Vec<Vec<Primitive>>,
    midis: Vec<Vec<MidiBundle>>,
    streams: Vec<Vec<Vec<f32>>>,
}

impl OutputsNode {
    pub fn get_streams(&self) -> &Vec<Vec<Vec<f32>>> {
        &self.streams
    }

    pub fn get_midis(&self) -> &Vec<Vec<MidiBundle>> {
        &self.midis
    }

    pub fn get_values(&self) -> &Vec<Vec<Primitive>> {
        &self.values
    }
}

impl NodeRuntime for OutputsNode {
    fn process<'a>(
        &mut self,
        context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        midi_store: &mut MidiStoreInterface,
        resources: &[&Resource],
    ) -> NodeResult<()> {
        for (socket_in, local_in) in ins.midis().zip(self.midis.iter_mut()) {
            for (channel_in, local_channel_in) in socket_in.iter().zip(local_in.iter_mut()) {
                local_channel_in.clear();

                if let Some(midi) = channel_in {
                    local_channel_in.clone_from_slice(midi_store.borrow_midi(midi).unwrap());
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
        let channels = default_channels(params.props, params.default_channel_count);

        let buffer_size = params.sound_config.buffer_size;

        if let Some(Property::SocketList(sockets)) = params.props.get("socket_list") {
            let midi_outputs = sockets
                .iter()
                .filter(|output| output.socket_type() == SocketType::Midi)
                .count();

            let value_outputs = sockets
                .iter()
                .filter(|output| output.socket_type() == SocketType::Value)
                .count();

            let stream_outputs = sockets
                .iter()
                .filter(|output| output.socket_type() == SocketType::Stream)
                .count();

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

    fn get_io(context: &NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
        if let Some(Property::SocketList(sockets)) = props.get("socket_list") {
            NodeIo::simple(
                sockets
                    .iter()
                    .map(|socket_type| NodeRow::from_type_and_direction(socket_type.clone(), SocketDirection::Input))
                    .collect::<Vec<NodeRow>>(),
            )
        } else {
            NodeIo::simple(vec![])
        }
    }
}
