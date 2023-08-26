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

    fn process(
        &mut self,
        _globals: NodeProcessGlobals,
        ins: Ins,
        outs: Outs,
        _resources: &[Option<(ResourceIndex, &dyn Any)>],
    ) -> NodeResult<()> {
        if let Some(frequency) = ins.values[0].as_ref().and_then(|x| x.as_float()) {
            self.oscillator.set_frequency(frequency.clamp(1.0, 20_000.0));
        }

        for frame in outs.streams[0].iter_mut() {
            *frame = self.oscillator.process();
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

    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            value_input(register("frequency"), Primitive::Float(440.0)),
            stream_output(register("audio")),
            NodeRow::Property(
                "waveform".to_string(),
                PropertyType::MultipleChoice(vec![
                    "sine".to_string(),
                    "sawtooth".to_string(),
                    "square".to_string(),
                    "triangle".to_string(),
                ]),
                Property::MultipleChoice("square".to_string()),
            ),
        ])
    }
}
