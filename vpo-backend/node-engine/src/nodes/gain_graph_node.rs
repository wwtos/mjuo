use serde::{Deserialize, Serialize};

use crate::connection::{SocketType, StreamSocketType, ValueType};
use crate::node::Node;

#[derive(Debug, Serialize, Deserialize)]
pub struct GainGraphNode {}

impl GainGraphNode {
    pub fn new() -> Self {
        GainGraphNode {}
    }
}

impl Node for GainGraphNode {
    fn list_input_sockets(&self) -> Vec<SocketType> {
        vec![
            SocketType::Stream(StreamSocketType::Audio),
            SocketType::Value(ValueType::Gain),
        ]
    }

    fn list_output_sockets(&self) -> Vec<SocketType> {
        vec![SocketType::Stream(StreamSocketType::Audio)]
    }
}
