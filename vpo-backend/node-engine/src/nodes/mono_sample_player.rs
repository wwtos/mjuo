use sound_engine::node::mono_buffer_player::MonoBufferPlayer;

use crate::{
    connection::{Primitive, StreamSocketType, ValueSocketType},
    errors::{NodeError, NodeOk},
    node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow},
};

#[derive(Debug)]
pub struct MonoSamplePlayerNode {
    player: MonoBufferPlayer,
    playing: bool,
}

impl Node for MonoSamplePlayerNode {
    fn init(&mut self, _state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        InitResult::simple(vec![
            NodeRow::ValueInput(ValueSocketType::Default, Primitive::Boolean(false)),
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0),
        ])
    }

    fn process(&mut self, state: NodeProcessState) -> Result<NodeOk<()>, NodeError> {
        NodeOk::no_warnings(())
    }

    fn accept_value_input(&mut self, _socket_type: &ValueSocketType, value: Primitive) {
        if let Some(engaged) = value.as_boolean() {
            if engaged {
                self.player.seek(0.0);
                self.playing = true;
            } else {
                self.playing = false;
            }
        }
    }
}
