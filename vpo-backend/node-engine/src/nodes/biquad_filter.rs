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

    fn process(
        &mut self,
        globals: NodeProcessGlobals,
        ins: Ins,
        outs: Outs,
        resources: &[(ResourceIndex, &dyn Any)],
    ) -> NodeResult<()> {
        if let Some(frequency) = ins.values[0].and_then(|f| f.as_float()) {
            self.filter.set_frequency(frequency.max(1.0));
        }

        if let Some(resonance) = ins.values[1].and_then(|r| r.as_float()) {
            self.filter.set_q(resonance);
        }

        for (frame_in, frame_out) in ins.streams[0].iter().zip(outs.streams[0].iter_mut()) {
            *frame_out = self.filter.filter_audio(*frame_in);
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
                stream_input(register("audio")),
                value_input(register("frequency"), Primitive::Float(20000.0)),
                value_input(register("resonance"), Primitive::Float(0.707)),
                stream_output(register("audio")),
            ],
            child_graph_io: None,
        }
    }
}
