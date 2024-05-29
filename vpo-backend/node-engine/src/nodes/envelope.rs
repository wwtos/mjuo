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

    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        _osc_store: &mut OscStore,
        _resources: &[Resource],
    ) {
        ins.value(0)[0].as_boolean().map(|gate| self.gate = gate);
        ins.value(1)[0].as_float().map(|attack| self.envelope.attack = attack);
        ins.value(2)[0].as_float().map(|decay| self.envelope.decay = decay);
        ins.value(3)[0]
            .as_float()
            .map(|sustain| self.envelope.sustain = sustain);
        ins.value(4)[0]
            .as_float()
            .map(|release| self.envelope.release = release);

        if !self.envelope.is_done() || self.gate {
            outs.value(0)[0] = float(self.envelope.process(self.gate));
        }
    }

    fn reset(&mut self) {
        self.gate = false;
        self.envelope.reset();
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

    fn get_io(_context: NodeGetIoContext, _props: SeaHashMap<String, Property>) -> NodeIo {
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
