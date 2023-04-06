use sound_engine::node::envelope::Envelope;
use sound_engine::SoundConfig;

use crate::nodes::prelude::*;

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

impl NodeRuntime for EnvelopeNode {
    fn init(&mut self, state: NodeInitState, child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        InitResult::nothing()
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

impl Node for EnvelopeNode {
    fn get_io(props: HashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            value_input("gate", Primitive::Boolean(false)),
            stream_output("gain", 0.0),
            value_input("attack", Primitive::Float(0.01)),
            value_input("decay", Primitive::Float(0.3)),
            value_input("sustain", Primitive::Float(0.8)),
            value_input("release", Primitive::Float(0.5)),
        ])
    }
}
