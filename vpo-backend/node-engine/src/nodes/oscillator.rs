use serde::{Deserialize, Serialize};
use sound_engine::node::oscillator::Oscillator;
use sound_engine::node::oscillator::Waveform;

use crate::connection::{Parameter, SocketType, StreamSocketType, ValueSocketType};
use crate::node::Node;

#[derive(Debug, Serialize, Deserialize)]
pub struct OscillatorNode {
    oscillator: Oscillator,
    audio_out: f32,
}

impl Default for OscillatorNode {
    fn default() -> Self {
        OscillatorNode {
            oscillator: Oscillator::new(Waveform::Square),
            audio_out: 0_f32,
        }
    }
}

impl Node for OscillatorNode {
    fn list_input_sockets(&self) -> Vec<SocketType> {
        vec![SocketType::Value(ValueSocketType::Frequency)]
    }

    fn process(&mut self) {
        self.audio_out = self.oscillator.process_fast();
    }

    fn accept_value_input(&mut self, socket_type: ValueSocketType, value: Parameter) {
        if socket_type == ValueSocketType::Frequency {
            self.oscillator.set_frequency(value.as_float().unwrap());
        }
    }

    fn get_stream_output(&self, socket_type: StreamSocketType) -> f32 {
        match socket_type {
            StreamSocketType::Audio => self.audio_out,
            _ => 0_f32,
        }
    }

    fn list_output_sockets(&self) -> Vec<SocketType> {
        vec![SocketType::Stream(StreamSocketType::Audio)]
    }
}
