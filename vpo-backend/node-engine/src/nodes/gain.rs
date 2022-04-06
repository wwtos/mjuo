use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::connection::{SocketType, StreamSocketType, ValueSocketType, Parameter};
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
    fn accept_stream_input(&mut self, _socket_type: StreamSocketType, value: f32) {
        self.value = value;
    }

    fn get_stream_output(&self, _socket_type: StreamSocketType) -> f32 {
        self.value * self.gain
    }

    fn accept_value_input(&mut self, _socket_type: ValueSocketType, value: Parameter) {
        if let Some(gain) = value.as_float() {
            self.gain = gain;
        }
    }

    fn init(
        &mut self,
        properties: &HashMap<String, Property>,
    ) -> (bool, Option<HashMap<String, Property>>) {
        if let Some(gain_prop) = properties.get("Default gain") {
            if let Property::Float(gain) = gain_prop {
                self.gain = gain.clamp(0.0, 1.0);
            }
        }

        (false, None)
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

    fn list_properties(&self) -> HashMap<String, PropertyType> {
        let mut props = HashMap::new();

        props.insert("Default gain".to_string(), PropertyType::Float);

        props
    }
}
