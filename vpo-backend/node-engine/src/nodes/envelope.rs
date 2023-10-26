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

    fn process<'a, 'arena: 'a, 'brand>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a, 'arena, 'brand>,
        outs: Outs<'a, 'arena, 'brand>,
        token: &mut GhostToken<'brand>,
        arena: &'arena BuddyArena,
        resources: &[&Resource],
    ) -> NodeResult<()> {
        if let Some(gate) = ins.values[0][0].get().as_boolean() {
            self.gate = gate;
        }

        if let Some(attack) = ins.values[1][0].get().as_float() {
            self.envelope.attack = attack;
        }

        if let Some(decay) = ins.values[2][0].get().as_float() {
            self.envelope.decay = decay;
        }

        if let Some(sustain) = ins.values[3][0].get().as_float() {
            self.envelope.sustain = sustain;
        }

        if let Some(release) = ins.values[4][0].get().as_float() {
            self.envelope.release = release;
        }

        if !self.envelope.is_done() || self.gate {
            outs.values[0][0].set(float(self.envelope.process(self.gate)));
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

    fn get_io(context: &NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            value_input("gate", Primitive::Boolean(false), 1),
            value_output("gain", 1),
            value_input("attack", Primitive::Float(0.01), 1),
            value_input("decay", Primitive::Float(0.3), 1),
            value_input("sustain", Primitive::Float(0.8), 1),
            value_input("release", Primitive::Float(0.5), 1),
        ])
    }
}
