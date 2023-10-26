use sound_engine::node::oscillator::Oscillator;
use sound_engine::node::oscillator::Waveform;

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct OscillatorNode {
    oscillator: Oscillator,
}

impl NodeRuntime for OscillatorNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        if let Some(waveform) = params.props.get("waveform") {
            let last_phase = self.oscillator.get_phase();

            self.oscillator = Oscillator::new(
                Waveform::from_string(&waveform.to_owned().as_multiple_choice().unwrap()).unwrap(),
                params.sound_config.sample_rate,
            );
            self.oscillator.set_phase(last_phase);
        }

        InitResult::nothing()
    }

    fn process<'a, 'arena: 'a, 'brand>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a, 'arena, 'brand>,
        outs: Outs<'a, 'arena, 'brand>,
        _token: &mut GhostToken<'brand>,
        _arena: &'arena BuddyArena,
        _resources: &[&Resource],
    ) -> NodeResult<()> {
        if let Some(frequency) = ins.values[0][0].get().as_float() {
            self.oscillator.set_frequency(frequency.clamp(1.0, 20_000.0));
        }

        for frame in outs.streams[0][0].iter() {
            frame.set(self.oscillator.process());
        }

        NodeOk::no_warnings(())
    }
}

impl Node for OscillatorNode {
    fn new(sound_config: &SoundConfig) -> Self {
        OscillatorNode {
            oscillator: Oscillator::new(Waveform::Square, sound_config.sample_rate),
        }
    }

    fn get_io(_context: &NodeGetIoContext, _props: HashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            value_input("frequency", Primitive::Float(440.0), 1),
            stream_output("audio", 1),
            multiple_choice("waveform", &["sine", "sawtooth", "square", "triangle"], "square"),
        ])
    }
}
