use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sound_engine::node::envelope::Envelope;
use sound_engine::node::AudioNode;
use sound_engine::SoundConfig;

use crate::connection::{Primitive, StreamSocketType, ValueSocketType};
use crate::node::{Node, NodeRow, InitResult};
use crate::property::{Property, PropertyType};

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvelopeNode {
    envelope: Envelope,
    last_val: f32,
}

impl EnvelopeNode {
    pub fn new(config: &SoundConfig) -> Self {
        EnvelopeNode {
            envelope: Envelope::new(config, 0.02, 0.2, 0.8, 0.5),
            last_val: 0.0,
        }
    }
}

impl Node for EnvelopeNode {
    fn accept_value_input(&mut self, socket_type: ValueSocketType, value: Primitive) {
        if socket_type == ValueSocketType::Gate {
            if let Some(gate) = value.as_float() {
                self.envelope.set_gate(gate);
            }
        }
    }

    fn get_stream_output(&self, _socket_type: StreamSocketType) -> f32 {
        self.envelope.get_gain()
    }
    
    fn init(&mut self, properties: &HashMap<String, Property>) -> InitResult {
        if let Some(attack_raw) = properties.get("attack") {
            if let Property::Float(attack) = attack_raw {
                self.envelope.attack = *attack;
            }
        }

        if let Some(decay_raw) = properties.get("decay") {
            if let Property::Float(decay) = decay_raw {
                self.envelope.decay = *decay;
            }
        }

        if let Some(sustain_raw) = properties.get("sustain") {
            if let Property::Float(sustain) = sustain_raw {
                self.envelope.sustain = *sustain;
            }
        }

        if let Some(release_raw) = properties.get("release") {
            if let Property::Float(release) = release_raw {
                self.envelope.release = *release;
            }
        }

        InitResult {
            did_rows_change: false,
            node_rows: vec![
                NodeRow::ValueInput(ValueSocketType::Gate, Primitive::Boolean(false)),
                NodeRow::StreamOutput(StreamSocketType::Gain, 0.0),
                NodeRow::Property("attack".to_string(), PropertyType::Float, Property::Float(0.0)),
                NodeRow::Property("decay".to_string(), PropertyType::Float, Property::Float(0.0)),
                NodeRow::Property("sustain".to_string(), PropertyType::Float, Property::Float(0.0)),
                NodeRow::Property("release".to_string(), PropertyType::Float, Property::Float(0.0)),
            ],
            changed_properties: None
        }
    }

    fn process(&mut self) {
        self.envelope.process();
    }
}
