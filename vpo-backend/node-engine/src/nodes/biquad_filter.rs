use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sound_engine::SoundConfig;
use sound_engine::node::biquad_filter::{BiquadFilter, BiquadFilterType};

use crate::connection::{StreamSocketType, ValueSocketType, Primitive};
use crate::errors::ErrorsAndWarnings;
use crate::node::{InitResult, Node, NodeRow};
use crate::property::{Property, PropertyType};

#[derive(Debug, Serialize, Deserialize)]
pub struct BiquadFilterNode {
    filter: BiquadFilter
}

impl BiquadFilterNode {
    pub fn new(config: &SoundConfig) -> BiquadFilterNode {
        BiquadFilterNode {
            filter: BiquadFilter::new(config, BiquadFilterType::Lowpass, 20_000.0, 0.707)
        }
    }
}

impl Node for BiquadFilterNode {
    fn accept_value_input(&mut self, socket_type: ValueSocketType, value: Primitive) {
        match socket_type {
            ValueSocketType::Frequency => {
                if let Some(frequency) = value.as_float() {
                    self.filter.set_frequency(frequency);
                }
            }
            ValueSocketType::Resonance => {
                if let Some(resonance) = value.as_float() {
                    self.filter.set_q(resonance);
                }
            }
            _ => {}
        }
    }

    fn accept_stream_input(&mut self, socket_type: StreamSocketType, value: f32) {
        match socket_type {
            StreamSocketType::Audio => {
                self.filter.set_audio_in(value);
            }
            _ => {}
        };
    }

    fn process(&mut self) -> Result<(), ErrorsAndWarnings> {
        self.filter.process();

        Ok(())
    }

    fn get_stream_output(&self, _socket_type: StreamSocketType) -> f32 {
        self.filter.get_output_out()
    }

    fn init(&mut self, properties: &HashMap<String, Property>) -> InitResult {
        if let Some(Property::MultipleChoice(filter_type)) = properties.get("filter_type") {
            self.filter.set_filter_type(match filter_type.as_str() {
                "lowpass" => BiquadFilterType::Lowpass,
                "highpass" => BiquadFilterType::Highpass,
                "bandpass" => BiquadFilterType::Bandpass,
                "notch" => BiquadFilterType::Notch,
                "allpass" => BiquadFilterType::Allpass,
                _ => unreachable!("Type passed in was not a multiple choice option!")
            });
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
            NodeRow::StreamInput(StreamSocketType::Audio, 0.0),
            NodeRow::ValueInput(ValueSocketType::Frequency, Primitive::Float(20000.0)),
            NodeRow::ValueInput(ValueSocketType::Resonance, Primitive::Float(0.707)),
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0),
        ])
    }
}
