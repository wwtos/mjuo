use std::collections::HashMap;

use sound_engine::node::envelope::Envelope;
use sound_engine::node::AudioNode;
use sound_engine::SoundConfig;

use crate::connection::{Primitive, StreamSocketType, ValueSocketType};
use crate::errors::ErrorsAndWarnings;
use crate::node::{InitResult, Node, NodeRow};
use crate::property::Property;
use crate::socket_registry::SocketRegistry;

#[derive(Debug)]
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
    fn accept_value_input(&mut self, socket_type: &ValueSocketType, value: Primitive) {
        match socket_type {
            ValueSocketType::Gate => {
                if let Some(gate) = value.as_float() {
                    self.envelope.set_gate(gate);
                }
            }
            ValueSocketType::Attack => {
                if let Some(attack) = value.as_float() {
                    self.envelope.attack = attack;
                }
            }
            ValueSocketType::Decay => {
                if let Some(decay) = value.as_float() {
                    self.envelope.decay = decay;
                }
            }
            ValueSocketType::Sustain => {
                if let Some(sustain) = value.as_float() {
                    self.envelope.sustain = sustain;
                }
            }
            ValueSocketType::Release => {
                if let Some(release) = value.as_float() {
                    self.envelope.release = release;
                }
            }
            _ => {}
        }
    }

    fn get_stream_output(&self, _socket_type: &StreamSocketType) -> f32 {
        self.envelope.get_gain()
    }

    fn init(&mut self, _properties: &HashMap<String, Property>, _registry: &mut SocketRegistry) -> InitResult {
        InitResult::simple(vec![
            NodeRow::ValueInput(ValueSocketType::Gate, Primitive::Boolean(false)),
            NodeRow::StreamOutput(StreamSocketType::Gain, 0.0),
            NodeRow::ValueInput(ValueSocketType::Attack, Primitive::Float(0.01)),
            NodeRow::ValueInput(ValueSocketType::Decay, Primitive::Float(0.3)),
            NodeRow::ValueInput(ValueSocketType::Sustain, Primitive::Float(0.8)),
            NodeRow::ValueInput(ValueSocketType::Release, Primitive::Float(0.5)),
        ])
    }

    fn process(&mut self) -> Result<(), ErrorsAndWarnings> {
        self.envelope.process();

        Ok(())
    }
}
