use serde::{Deserialize, Serialize};

use crate::connection::{SocketType, StreamSocketType, ValueSocketType};
use crate::node::Node;

#[derive(Debug, Serialize, Deserialize)]
pub struct GainGraphNode {
    gain: f32,
    value: f32,
}

impl Default for GainGraphNode {
    fn default() -> Self {
        GainGraphNode {
            gain: 0.2,
            value: 0.0,
        }
    }
}

impl Node for GainGraphNode {
    fn accept_stream_input(&mut self, _socket_type: StreamSocketType, value: f32) {
        self.value = value;
    }

    fn get_stream_output(&self, _socket_type: StreamSocketType) -> f32 {
        self.value * self.gain
    }

    fn list_input_sockets(&self) -> Vec<SocketType> {
        vec![
            SocketType::Stream(StreamSocketType::Audio),
            SocketType::Value(ValueSocketType::Gain),
        ]
    }

    fn list_output_sockets(&self) -> Vec<SocketType> {
        vec![SocketType::Stream(StreamSocketType::Audio)]
    }
}
