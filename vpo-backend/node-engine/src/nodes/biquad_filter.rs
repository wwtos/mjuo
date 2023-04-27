use std::collections::HashMap;

use sound_engine::node::biquad_filter::{BiquadFilter, BiquadFilterType};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct BiquadFilterNode {
    filter: BiquadFilter,
}

impl NodeRuntime for BiquadFilterNode {
    fn init(&mut self, state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
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

        InitResult::nothing()
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
        streams_in: &[&[f32]],
        streams_out: &mut [&mut [f32]],
    ) -> Result<NodeOk<()>, NodeError> {
        for i in 0..streams_in[0].len() {
            streams_out[0][i] = self.filter.filter_audio(streams_in[0][i]);
        }

        NodeOk::no_warnings(())
    }
}

impl Node for BiquadFilterNode {
    fn new(config: &SoundConfig) -> BiquadFilterNode {
        BiquadFilterNode {
            filter: BiquadFilter::new(config, BiquadFilterType::Lowpass, 20_000.0, 0.707),
        }
    }

    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo {
            node_rows: vec![
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
                stream_input(register("audio"), 0.0),
                value_input(register("frequency"), Primitive::Float(20000.0)),
                value_input(register("resonance"), Primitive::Float(0.707)),
                stream_output(register("audio"), 0.0),
            ],
            child_graph_io: None,
        }
    }
}
