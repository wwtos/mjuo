use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::connection::StreamSocketType;
use crate::node::{InitResult, Node, NodeRow};
use crate::property::Property;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    fn accept_stream_input(&mut self, socket_type: StreamSocketType, value: f32) {
        match socket_type {
            StreamSocketType::Audio => self.value = value,
            StreamSocketType::Gain => self.gain = value,
            _ => {}
        };
    }

    fn get_stream_output(&self, _socket_type: StreamSocketType) -> f32 {
        self.value * self.gain
    }

    fn init(&mut self, properties: &HashMap<String, Property>) -> InitResult {
        if let Some(Property::Float(gain)) = properties.get("default_gain") {
            self.gain = gain.clamp(0.0, 1.0);
        }

        InitResult {
            did_rows_change: false,
            node_rows: vec![
                NodeRow::StreamInput(StreamSocketType::Audio, 0.0),
                NodeRow::StreamInput(StreamSocketType::Gain, 0.0),
                NodeRow::StreamOutput(StreamSocketType::Audio, 0.0),
            ],
            changed_properties: None,
        }
    }
}
