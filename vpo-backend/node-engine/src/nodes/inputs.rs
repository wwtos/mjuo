use std::iter::repeat_with;

use crate::nodes::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct InputsNode {
    values: Vec<Primitive>,
    midis: Vec<MidiChannel>,
    streams: Vec<Vec<Vec<f32>>>,
    sent: bool,
}

impl InputsNode {
    pub fn set_values(&mut self, values: Vec<Primitive>) {
        self.values = values;
        self.sent = false;
    }

    pub fn set_midis(&mut self, midis: Vec<MidiChannel>) {
        self.midis = midis;
        self.sent = false;
    }
    pub fn streams_mut(&mut self) -> &mut Vec<Vec<Vec<f32>>> {
        &mut self.streams
    }
}

impl NodeRuntime for InputsNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        if let Some(Property::SocketList(sockets)) = params.props.get("socket_list") {
            self.streams = sockets
                .iter()
                .filter_map(|socket| {
                    if socket.socket_type() == SocketType::Stream {
                        Some(
                            repeat_with(|| vec![0.0; params.sound_config.buffer_size])
                                .take(socket.channels())
                                .collect(),
                        )
                    } else {
                        None
                    }
                })
                .collect();
        }

        InitResult::nothing()
    }

    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        _ins: Ins<'a>,
        mut outs: Outs<'a>,
        midi_store: &mut MidiStoreInterface,
        _resources: &[Resource],
    ) -> NodeResult<()> {
        if !self.sent {
            for (mut midi_socket, message_in) in outs.midis().zip(self.midis.drain(..)) {
                midi_socket[0] = midi_store.register_midis(message_in.into_iter());
            }

            for (mut values_out, value_to_output) in outs.values().zip(self.values.iter()) {
                values_out[0] = *value_to_output;
            }

            for (mut socket_out, socket) in outs.streams().zip(self.streams.iter_mut()) {
                for (channel_out, channel) in socket_out.iter_mut().zip(socket.iter_mut()) {
                    channel_out.copy_from_slice(&channel[..]);
                }
            }

            self.sent = true;
        }

        NodeOk::no_warnings(())
    }
}

impl Node for InputsNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        InputsNode {
            values: vec![],
            midis: vec![],
            streams: vec![],
            sent: false,
        }
    }

    fn get_io(_context: &NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
        if let Some(Property::SocketList(sockets)) = props.get("socket_list") {
            NodeIo::simple(
                sockets
                    .iter()
                    .map(|socket| NodeRow::from_type_and_direction(socket.clone(), SocketDirection::Output))
                    .collect::<Vec<NodeRow>>(),
            )
        } else {
            NodeIo::simple(vec![])
        }
    }
}
