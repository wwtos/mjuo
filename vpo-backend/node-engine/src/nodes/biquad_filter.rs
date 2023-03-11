use sound_engine::node::biquad_filter::{BiquadFilter, BiquadFilterType};
use sound_engine::SoundConfig;

use crate::connection::{Primitive, StreamSocketType, ValueSocketType};
use crate::errors::{NodeError, NodeOk};
use crate::node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow};
use crate::property::{Property, PropertyType};

#[derive(Debug, Clone)]
pub struct BiquadFilterNode {
    filter: BiquadFilter,
}

impl BiquadFilterNode {
    pub fn new(config: &SoundConfig) -> BiquadFilterNode {
        BiquadFilterNode {
            filter: BiquadFilter::new(config, BiquadFilterType::Lowpass, 20_000.0, 0.707),
        }
    }
}

impl Node for BiquadFilterNode {
    fn init(&mut self, state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        self.filter.reset();

        if let Some(Property::MultipleChoice(filter_type)) = state.props.get("filter_type") {
            self.filter.set_filter_type(match filter_type.as_str() {
                "lowpass" => BiquadFilterType::Lowpass,
                "highpass" => BiquadFilterType::Highpass,
                "bandpass" => BiquadFilterType::Bandpass,
                "notch" => BiquadFilterType::Notch,
                "allpass" => BiquadFilterType::Allpass,
                _ => unimplemented!("Type passed in was not a multiple choice option!"),
            });
        } else {
            self.filter.set_filter_type(BiquadFilterType::Lowpass);
        }

        InitResult::simple(vec![
            NodeRow::Property(
                "filter_type".to_string(),
                PropertyType::MultipleChoice(vec![
                    "lowpass".to_string(),
                    "highpass".to_string(),
                    "bandpass".to_string(),
                    "notch".to_string(),
                    "allpass".to_string(),
                ]),
                Property::MultipleChoice("lowpass".to_string()),
            ),
            NodeRow::StreamInput(StreamSocketType::Audio, 0.0, false),
            NodeRow::ValueInput(ValueSocketType::Frequency, Primitive::Float(20000.0), false),
            NodeRow::ValueInput(ValueSocketType::Resonance, Primitive::Float(0.707), false),
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0, false),
        ])
    }

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        if let [frequency, resonance] = &values_in {
            if let Some(frequency) = frequency.clone().and_then(|f| f.as_float()) {
                self.filter.set_frequency(frequency.max(1.0));
            }

            if let Some(resonance) = resonance.clone().and_then(|r| r.as_float()) {
                self.filter.set_q(resonance);
            }
        }
    }

    fn process(
        &mut self,
        _state: NodeProcessState,
        streams_in: &[f32],
        streams_out: &mut [f32],
    ) -> Result<NodeOk<()>, NodeError> {
        streams_out[0] = self.filter.filter_audio(streams_in[0]);

        NodeOk::no_warnings(())
    }
}
