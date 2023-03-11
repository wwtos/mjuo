use sound_engine::node::envelope::Envelope;
use sound_engine::SoundConfig;

use crate::connection::{Primitive, StreamSocketType, ValueSocketType};
use crate::errors::{NodeError, NodeOk};
use crate::node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow};
#[derive(Debug, Clone)]
pub struct EnvelopeNode {
    envelope: Envelope,
    gate: f32,
}

impl EnvelopeNode {
    pub fn new(config: &SoundConfig) -> Self {
        EnvelopeNode {
            envelope: Envelope::new(config, 0.02, 0.2, 0.8, 0.5),
            gate: 0.0,
        }
    }
}

impl Node for EnvelopeNode {
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

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        if let [gate, attack, decay, sustain, release] = values_in {
            if let Some(gate) = gate.clone().and_then(|gate| gate.as_float()) {
                self.gate = gate;
            }

            if let Some(attack) = attack.clone().and_then(|attack| attack.as_float()) {
                self.envelope.attack = attack;
            }

            if let Some(decay) = decay.clone().and_then(|decay| decay.as_float()) {
                self.envelope.decay = decay;
            }

            if let Some(sustain) = sustain.clone().and_then(|sustain| sustain.as_float()) {
                self.envelope.sustain = sustain;
            }

            if let Some(release) = release.clone().and_then(|release| release.as_float()) {
                self.envelope.release = release;
            }
        }
    }

    fn process(
        &mut self,
        _state: NodeProcessState,
        _streams_in: &[f32],
        streams_out: &mut [f32],
    ) -> Result<NodeOk<()>, NodeError> {
        streams_out[0] = self.envelope.process(self.gate);

        NodeOk::no_warnings(())
    }
}
