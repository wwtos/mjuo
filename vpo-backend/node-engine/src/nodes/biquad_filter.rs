use std::collections::HashMap;

use itertools::multizip;
use sound_engine::node::filter::{filter_coeffs, BiquadFilter, FilterSpec, FilterType};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct BiquadFilterNode {
    filters: Vec<BiquadFilter>,
    filter_spec: FilterSpec<f32>,
    q: f32,
}

impl BiquadFilterNode {
    fn recompute(&mut self) {
        let coeffs = filter_coeffs(self.filter_spec.clone());

        for filter in self.filters.iter_mut() {
            filter.set_coeffs(coeffs.clone());
        }
    }
}

impl NodeRuntime for BiquadFilterNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
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

        self.filters.resize(
            default_channels(&params.props, params.default_channel_count),
            BiquadFilter::default(),
        );

        self.recompute();

        InitResult::nothing()
    }

    fn process<'brand>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'_, 'brand>,
        outs: Outs<'_, 'brand>,
        token: &mut GhostToken<'brand>,
        resources: &[&dyn Any],
    ) -> NodeResult<()> {
        if let Some(frequency) = ins.values[0][0].borrow(token).as_float() {
            self.filter_spec.f0 = frequency.max(1.0);
            self.recompute();
        }

        if let Some(resonance) = ins.values[1][0].borrow(token).as_float() {
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

        for (channel_in, channel_out, filter) in multizip((ins.streams[0], outs.streams[0], self.filters.iter_mut())) {
            for (frame_in, frame_out) in channel_in.iter().zip(channel_out.iter()) {
                *frame_out.borrow_mut(token) = filter.filter_sample(*frame_in.borrow(token));
            }
        }

        NodeOk::no_warnings(())
    }
}

impl Node for BiquadFilterNode {
    fn new(_config: &SoundConfig) -> BiquadFilterNode {
        BiquadFilterNode {
            filters: vec![],
            filter_spec: FilterSpec::none(),
            q: 0.7,
        }
    }

    fn get_io(context: NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
        let polyphony = default_channels(&props, context.default_channel_count);

        NodeIo {
            node_rows: vec![
                with_channels(context.default_channel_count),
                multiple_choice(
                    "filter_type",
                    &["lowpass", "highpass", "bandpass", "notch", "allpass"],
                    "lowpass",
                ),
                stream_input("audio", polyphony),
                value_input("frequency", Primitive::Float(20000.0), 1),
                value_input("resonance", Primitive::Float(0.707), 1),
                stream_output("audio", polyphony),
            ],
            child_graph_io: None,
        }
    }
}
