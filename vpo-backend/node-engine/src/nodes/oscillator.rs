use sound_engine::node::oscillator::Oscillator;
use sound_engine::node::oscillator::Waveform;

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct OscillatorNode {
    oscillator: Oscillator,
}

impl Default for OscillatorNode {
    fn default() -> Self {
        OscillatorNode {
            oscillator: Oscillator::new(Waveform::Square),
        }
    }
}

impl NodeRuntime for OscillatorNode {
    fn init(&mut self, state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        if let Some(waveform) = state.props.get("waveform") {
            let last_phase = self.oscillator.get_phase();

            self.oscillator =
                Oscillator::new(Waveform::from_string(&waveform.to_owned().as_multiple_choice().unwrap()).unwrap());
            self.oscillator.set_phase(last_phase);
        }

        InitResult::nothing()
    }

    fn process(
        &mut self,
        _state: NodeProcessState,
        _streams_in: &[&[f32]],
        streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        for frame in streams_out[0].iter_mut() {
            *frame = self.oscillator.process();
        }

        NodeOk::no_warnings(())
    }

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        if let [Some(frequency)] = values_in {
            if let Some(frequency) = frequency.clone().as_float() {
                self.oscillator.set_frequency(frequency.clamp(1.0, 20_000.0));
            }
        }
    }
}

impl Node for OscillatorNode {
    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            value_input(register("frequency"), Primitive::Float(440.0)),
            stream_output(register("audio"), 0.0),
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
