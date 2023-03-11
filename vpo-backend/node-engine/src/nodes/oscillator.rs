use sound_engine::node::oscillator::Oscillator;
use sound_engine::node::oscillator::Waveform;

use crate::connection::{Primitive, StreamSocketType, ValueSocketType};
use crate::errors::NodeError;
use crate::errors::NodeOk;
use crate::node::InitResult;
use crate::node::Node;
use crate::node::NodeInitState;
use crate::node::NodeProcessState;
use crate::node::NodeRow;
use crate::property::Property;
use crate::property::PropertyType;

#[derive(Debug, Clone)]
pub struct OscillatorNode {
    oscillator: Oscillator,
    audio_out: f32,
}

impl Default for OscillatorNode {
    fn default() -> Self {
        OscillatorNode {
            oscillator: Oscillator::new(Waveform::Square),
            audio_out: 0_f32,
        }
    }
}

impl Node for OscillatorNode {
    fn init(&mut self, state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        if let Some(waveform) = state.props.get("waveform") {
            let last_phase = self.oscillator.get_phase();

            self.oscillator =
                Oscillator::new(Waveform::from_string(&waveform.to_owned().as_multiple_choice().unwrap()).unwrap());
            self.oscillator.set_phase(last_phase);
        }

        InitResult::simple(vec![
            NodeRow::ValueInput(ValueSocketType::Frequency, Primitive::Float(440.0), false),
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0, false),
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

    fn process(
        &mut self,
        _state: NodeProcessState,
        _streams_in: &[f32],
        streams_out: &mut [f32],
    ) -> Result<NodeOk<()>, NodeError> {
        streams_out[0] = self.oscillator.process();

        NodeOk::no_warnings(())
    }

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        if let [Some(frequency)] = values_in {
            if let Some(frequency) = frequency.clone().as_float() {
                self.oscillator.set_frequency(frequency);
            }
        }
    }
}
