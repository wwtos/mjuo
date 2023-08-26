use std::collections::HashMap;

use sound_engine::node::filter::{filter_coeffs, BiquadFilter, FilterSpec, FilterType};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct BiquadFilterNode {
    filter: BiquadFilter,
    filter_spec: FilterSpec<f32>,
    q: f32,
}

impl BiquadFilterNode {
    fn recompute(&mut self) {
        self.filter.set_coeffs(filter_coeffs(self.filter_spec.clone()));
    }
}

impl NodeRuntime for BiquadFilterNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        self.filter.reset_history();

        if let Some(Property::MultipleChoice(filter_type)) = params.props.get("filter_type") {
            self.filter_spec = FilterSpec {
                f0: self.filter_spec.f0,
                fs: params.sound_config.sample_rate as f32,
                filter_type: match filter_type.as_str() {
                    "lowpass" => FilterType::LowPass { q: self.q },
                    "highpass" => FilterType::HighPass { q: self.q },
                    "bandpass" => FilterType::BandPass { bandwidth: self.q },
                    "notch" => FilterType::Notch { bandwidth: self.q },
                    "allpass" => FilterType::AllPass { q: self.q },
                    _ => unimplemented!("Type passed in was not a multiple choice option!"),
                },
            };
        }

        self.filter.set_coeffs(filter_coeffs(self.filter_spec.clone()));

        InitResult::nothing()
    }

    fn process(
        &mut self,
        _globals: NodeProcessGlobals,
        ins: Ins,
        outs: Outs,
        _resources: &[Option<(ResourceIndex, &dyn Any)>],
    ) -> NodeResult<()> {
        if let Some(frequency) = ins.values[0].as_ref().and_then(|f| f.as_float()) {
            self.filter_spec.f0 = frequency.max(1.0);
            self.recompute();
        }

        if let Some(resonance) = ins.values[1].as_ref().and_then(|r| r.as_float()) {
            match &mut self.filter_spec.filter_type {
                FilterType::LowPass { q } | FilterType::HighPass { q } | FilterType::AllPass { q } => {
                    *q = resonance;
                }
                FilterType::Notch { bandwidth }
                | FilterType::BandPass { bandwidth }
                | FilterType::Peaking { bandwidth, .. } => {
                    *bandwidth = resonance;
                }
                FilterType::LowShelf { slope, .. } | FilterType::HighShelf { slope, .. } => {
                    *slope = resonance;
                }
                FilterType::None => {}
            }
        }

        for (frame_in, frame_out) in ins.streams[0].iter().zip(outs.streams[0].iter_mut()) {
            *frame_out = self.filter.filter_sample(*frame_in);
        }

        NodeOk::no_warnings(())
    }
}

impl Node for BiquadFilterNode {
    fn new(_config: &SoundConfig) -> BiquadFilterNode {
        BiquadFilterNode {
            filter: BiquadFilter::new(FilterSpec::none()),
            filter_spec: FilterSpec::none(),
            q: 0.7,
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
