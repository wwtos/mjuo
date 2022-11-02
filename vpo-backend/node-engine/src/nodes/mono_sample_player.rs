use asset_manager::AssetIndex;
use sound_engine::{node::mono_buffer_player::MonoBufferPlayer, SoundConfig};

use crate::{
    connection::{Primitive, StreamSocketType, ValueSocketType},
    errors::{NodeError, NodeOk},
    node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow},
};

#[derive(Debug, Clone)]
pub struct MonoSamplePlayerNode {
    player: Option<MonoBufferPlayer>,
    playing: bool,
    index: AssetIndex,
    config: SoundConfig,
    output: f32,
}

impl MonoSamplePlayerNode {
    pub fn new(config: &SoundConfig) -> Self {
        MonoSamplePlayerNode {
            player: None,
            playing: false,
            index: AssetIndex {
                index: 0,
                generation: 0,
            },
            config: config.clone(),
            output: 0.0,
        }
    }
}

impl Node for MonoSamplePlayerNode {
    fn init(&mut self, state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        self.index = state.global_state.samples.get_index("sample:060-C.wav").unwrap();

        if let None = self.player {
            let buffer = state.global_state.samples.borrow_asset(self.index).unwrap();

            self.player = Some(MonoBufferPlayer::new(&self.config, buffer));
        }

        InitResult::simple(vec![
            NodeRow::ValueInput(ValueSocketType::Default, Primitive::Boolean(false)),
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0),
        ])
    }

    fn process(&mut self, state: NodeProcessState) -> Result<NodeOk<()>, NodeError> {
        if let Some(player) = &mut self.player {
            if self.playing {
                let buffer = state.samples.borrow_asset(self.index).unwrap();
                self.output = player.get_next_sample(buffer);
            } else {
                self.output = 0.0;
            }
        }

        NodeOk::no_warnings(())
    }

    fn accept_value_input(&mut self, _socket_type: &ValueSocketType, value: Primitive) {
        if let Some(player) = &mut self.player {
            if let Some(engaged) = value.as_boolean() {
                if engaged {
                    player.seek(0.0);
                    self.playing = true;
                } else {
                    self.playing = false;
                }
            }
        }
    }

    fn get_stream_output(&self, _socket_type: &StreamSocketType) -> f32 {
        self.output
    }
}
