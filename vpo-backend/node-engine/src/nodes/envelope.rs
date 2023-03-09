use sound_engine::node::envelope::Envelope;
use sound_engine::node::AudioNode;
use sound_engine::SoundConfig;

use crate::connection::{Primitive, StreamSocketType, ValueSocketType};
use crate::errors::{NodeError, NodeOk};
use crate::node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow};
#[derive(Debug, Clone)]
pub struct EnvelopeNode {
    envelope: Envelope,
}

impl EnvelopeNode {
    pub fn new(config: &SoundConfig) -> Self {
        EnvelopeNode {
            envelope: Envelope::new(config, 0.02, 0.2, 0.8, 0.5),
        }
    }
}

impl Node for EnvelopeNode {
    fn accept_value_input(&mut self, socket_type: ValueSocketType, value: Primitive) {
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

    fn get_stream_output(&self, _socket_type: StreamSocketType) -> f32 {
        self.envelope.get_gain()
    }

    fn init(&mut self, _state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        InitResult::simple(vec![
            NodeRow::ValueInput(ValueSocketType::Gate, Primitive::Boolean(false), false),
            NodeRow::StreamOutput(StreamSocketType::Gain, 0.0, false),
            NodeRow::ValueInput(ValueSocketType::Attack, Primitive::Float(0.01), false),
            NodeRow::ValueInput(ValueSocketType::Decay, Primitive::Float(0.3), false),
            NodeRow::ValueInput(ValueSocketType::Sustain, Primitive::Float(0.8), false),
            NodeRow::ValueInput(ValueSocketType::Release, Primitive::Float(0.5), false),
        ])
    }

    fn process(&mut self, _state: NodeProcessState) -> Result<NodeOk<()>, NodeError> {
        self.envelope.process();

        NodeOk::no_warnings(())
    }
}
