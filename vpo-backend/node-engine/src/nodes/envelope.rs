use sound_engine::node::envelope::Envelope;
use sound_engine::SoundConfig;

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct EnvelopeNode {
    envelope: Envelope,
    gate: bool,
    out: Option<f32>,
}

impl NodeRuntime for EnvelopeNode {
    fn init(&mut self, _state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        InitResult::nothing()
    }

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        if let [gate, attack, decay, sustain, release] = values_in {
            if let Some(gate) = gate.clone().and_then(|gate| gate.as_boolean()) {
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
        _streams_in: &[&[f32]],
        _streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        if !self.envelope.is_done() || self.gate {
            self.out = Some(self.envelope.process(self.gate));
        }

        NodeOk::no_warnings(())
    }

    fn get_value_outputs(&mut self, values_out: &mut [Option<Primitive>]) {
        values_out[0] = self.out.take().map(Primitive::Float);
    }
}

impl Node for EnvelopeNode {
    fn new(config: &SoundConfig) -> Self {
        let samples_per_second = config.sample_rate as f32 / config.buffer_size as f32;

        EnvelopeNode {
            envelope: Envelope::new(samples_per_second, 0.02, 0.2, 0.8, 0.5),
            gate: false,
            out: None,
        }
    }

    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            value_input(register("gate"), Primitive::Boolean(false)),
            value_output(register("gain")),
            value_input(register("attack"), Primitive::Float(0.01)),
            value_input(register("decay"), Primitive::Float(0.3)),
            value_input(register("sustain"), Primitive::Float(0.8)),
            value_input(register("release"), Primitive::Float(0.5)),
        ])
    }
}
