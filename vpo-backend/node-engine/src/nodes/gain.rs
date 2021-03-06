use std::collections::HashMap;

use rhai::Engine;

use crate::connection::StreamSocketType;
use crate::node::{InitResult, Node, NodeRow};
use crate::property::Property;
use crate::socket_registry::SocketRegistry;

#[derive(Debug, Clone)]
pub struct GainGraphNode {
    gain: f32,
    value: f32,
}

impl Default for GainGraphNode {
    fn default() -> Self {
        GainGraphNode { gain: 0.2, value: 0.0 }
    }
}

impl Node for GainGraphNode {
    fn accept_stream_input(&mut self, socket_type: &StreamSocketType, value: f32) {
        match socket_type {
            StreamSocketType::Audio => self.value = value,
            StreamSocketType::Gain => self.gain = value,
            _ => {}
        };
    }

    fn get_stream_output(&self, _socket_type: &StreamSocketType) -> f32 {
        self.value * self.gain
    }

    fn init(
        &mut self,
        properties: &HashMap<String, Property>,
        _registry: &mut SocketRegistry,
        _scripting_engine: &Engine,
    ) -> InitResult {
        if let Some(Property::Float(gain)) = properties.get("default_gain") {
            self.gain = gain.clamp(0.0, 1.0);
        }

        InitResult::simple(vec![
            NodeRow::StreamInput(StreamSocketType::Audio, 0.0),
            NodeRow::StreamInput(StreamSocketType::Gain, 0.0),
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0),
        ])
    }
}
