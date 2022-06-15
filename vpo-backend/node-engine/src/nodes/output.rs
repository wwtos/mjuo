use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::connection::{StreamSocketType};
use crate::node::{Node, InitResult, NodeRow};
use crate::property::Property;

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
    fn init(&mut self, properties: &HashMap<String, Property>) -> InitResult {
        InitResult::simple(vec![
            NodeRow::StreamInput(StreamSocketType::Audio, 0.0)
        ])
    }

    fn accept_stream_input(&mut self, _socket_type: StreamSocketType, value: f32) {
        self.current_value = value;
    }

    fn get_stream_output(&self, _socket_type: StreamSocketType) -> f32 {
        self.current_value
    }
}
