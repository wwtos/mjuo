use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::connection::{StreamSocketType, SocketType};
use crate::errors::ErrorsAndWarnings;
use crate::node::{InitResult, Node, NodeRow};
use crate::property::{Property, PropertyType};
use crate::socket_registry::SocketRegistry;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MixerNode {
    input_count: i32,
    last_input_count: i32,
    input_sum: f32,
    output_audio: f32,
}

impl Default for MixerNode {
    fn default() -> Self {
        MixerNode {
            input_count: 2,
            last_input_count: 2,
            input_sum: 0.0,
            output_audio: 0.0,
        }
    }
}

impl Node for MixerNode {
    fn accept_stream_input(&mut self, _socket_type: &StreamSocketType, value: f32) {
        self.input_sum += value;
    }

    fn process(&mut self) -> Result<(), ErrorsAndWarnings> {
        self.output_audio = self.input_sum / self.input_count as f32;
        self.input_sum = 0.0;

        Ok(())
    }

    fn get_stream_output(&self, _socket_type: &StreamSocketType) -> f32 {
        self.output_audio
    }

    fn init(&mut self, properties: &HashMap<String, Property>, registry: &mut SocketRegistry) -> InitResult {
        if let Some(Property::Integer(input_count)) = properties.get("input_count") {
            self.input_count = *input_count;
        }

        let mut node_rows = vec![
            NodeRow::Property("input_count".to_string(), PropertyType::Integer, Property::Integer(2),
        )];
        let did_rows_change = self.input_count != self.last_input_count;

        for i in 0..self.input_count {
            node_rows.push(NodeRow::StreamInput(
                registry.register_socket(
                    format!("stream.mixer.{}", i),
                    SocketType::Stream(StreamSocketType::Audio),
                    "stream.mixer".to_string(),
                    Some(json! { i })
                ).unwrap().as_stream().unwrap(), 
                0.0)
            );
        }

        InitResult {
            did_rows_change,
            node_rows: node_rows,
            changed_properties: None,
        }
    }
}
