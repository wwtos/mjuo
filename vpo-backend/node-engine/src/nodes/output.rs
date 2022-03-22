use serde::{Deserialize, Serialize};

use crate::connection::{SocketType, StreamSocketType};
use crate::node::Node;

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputNode {
    current_value: f32,
}

impl Default for OutputNode {
    fn default() -> Self {
        OutputNode { current_value: 0.0 }
    }
}

impl Node for OutputNode {
    fn list_input_sockets(&self) -> Vec<SocketType> {
        vec![SocketType::Stream(StreamSocketType::Audio)]
    }

    fn accept_stream_input(&mut self, _socket_type: StreamSocketType, value: f32) {
        self.current_value = value;
    }

    fn get_stream_output(&self, _socket_type: StreamSocketType) -> f32 {
        self.current_value
    }
}
