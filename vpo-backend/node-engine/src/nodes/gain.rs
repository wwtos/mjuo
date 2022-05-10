use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::connection::{SocketType, StreamSocketType};
use crate::node::Node;
use crate::property::{Property, PropertyType};

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

    fn init(
        &mut self,
        properties: &HashMap<String, Property>,
    ) -> (bool, Option<HashMap<String, Property>>) {
        if let Some(gain_prop) = properties.get("default_gain") {
            if let Property::Float(gain) = gain_prop {
                self.gain = gain.clamp(0.0, 1.0);
            }
        }

        (false, None)
    }

    fn list_input_sockets(&self) -> Vec<SocketType> {
        vec![
            SocketType::Stream(StreamSocketType::Audio),
            SocketType::Stream(StreamSocketType::Gain),
        ]
    }

    fn list_output_sockets(&self) -> Vec<SocketType> {
        vec![SocketType::Stream(StreamSocketType::Audio)]
    }

    fn list_properties(&self) -> HashMap<String, PropertyType> {
        let mut props = HashMap::new();

        props.insert("default_gain".to_string(), PropertyType::Float);

        props
    }
}
