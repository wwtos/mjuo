use sound_engine::node::envelope::Envelope;
use sound_engine::SoundConfig;

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct EnvelopeNode {
    envelope: Envelope,
    gate: bool,
}

impl NodeRuntime for EnvelopeNode {
    fn init(&mut self, _params: NodeInitParams) -> NodeResult<InitResult> {
        InitResult::nothing()
    }

    fn process(
        &mut self,
        _globals: NodeProcessGlobals,
        ins: Ins,
        outs: Outs,
        _resources: &[Option<(ResourceIndex, &dyn Any)>],
    ) -> NodeResult<()> {
        if let Some(gate) = ins.values[0].as_ref().and_then(|gate| gate.as_boolean()) {
            self.gate = gate;
        }

        if let Some(attack) = ins.values[0].as_ref().and_then(|attack| attack.as_float()) {
            self.envelope.attack = attack;
        }

        if let Some(decay) = ins.values[0].as_ref().and_then(|decay| decay.as_float()) {
            self.envelope.decay = decay;
        }

        if let Some(sustain) = ins.values[0].as_ref().and_then(|sustain| sustain.as_float()) {
            self.envelope.sustain = sustain;
        }

        if let Some(release) = ins.values[0].as_ref().and_then(|release| release.as_float()) {
            self.envelope.release = release;
        }

        if !self.envelope.is_done() || self.gate {
            outs.values[0] = Some(Primitive::Float(self.envelope.process(self.gate)));
        }

        NodeOk::no_warnings(())
    }
}

impl Node for EnvelopeNode {
    fn new(config: &SoundConfig) -> Self {
        let samples_per_second = config.sample_rate as f32 / config.buffer_size as f32;

        EnvelopeNode {
            envelope: Envelope::new(samples_per_second, 0.02, 0.2, 0.8, 0.5),
            gate: false,
        }
    }

    fn get_io(_props: HashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            value_input("gate", Primitive::Boolean(false)),
            value_output("gain"),
            value_input("attack", Primitive::Float(0.01)),
            value_input("decay", Primitive::Float(0.3)),
            value_input("sustain", Primitive::Float(0.8)),
            value_input("release", Primitive::Float(0.5)),
        ])
    }
}
