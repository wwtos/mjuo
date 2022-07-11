use std::collections::HashMap;

use rhai::Engine;
use sound_engine::node::oscillator::Oscillator;
use sound_engine::node::oscillator::Waveform;

use crate::connection::{Primitive, StreamSocketType, ValueSocketType};
use crate::errors::ErrorsAndWarnings;
use crate::node::InitResult;
use crate::node::Node;
use crate::node::NodeRow;
use crate::node_graph::NodeGraph;
use crate::property::Property;
use crate::property::PropertyType;
use crate::socket_registry::SocketRegistry;
use crate::traversal::traverser::Traverser;

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
    fn init(
        &mut self,
        properties: &HashMap<String, Property>,
        _registry: &mut SocketRegistry,
        _scripting_engine: &Engine,
    ) -> InitResult {
        if let Some(waveform) = properties.get("waveform") {
            let last_phase = self.oscillator.get_phase();

            self.oscillator =
                Oscillator::new(Waveform::from_string(&waveform.to_owned().as_multiple_choice().unwrap()).unwrap());
            self.oscillator.set_phase(last_phase);
        }

        InitResult::simple(vec![
            NodeRow::ValueInput(ValueSocketType::Frequency, Primitive::Float(440.0)),
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0),
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
        _current_time: i64,
        _scripting_engine: &Engine,
        _inner_graph: Option<(&mut NodeGraph, &Traverser)>,
    ) -> Result<(), ErrorsAndWarnings> {
        self.audio_out = self.oscillator.process_fast();

        Ok(())
    }

    fn accept_value_input(&mut self, socket_type: &ValueSocketType, value: Primitive) {
        if socket_type == &ValueSocketType::Frequency {
            self.oscillator.set_frequency(value.as_float().unwrap());
        }
    }

    fn get_stream_output(&self, socket_type: &StreamSocketType) -> f32 {
        match socket_type {
            StreamSocketType::Audio => self.audio_out,
            _ => 0_f32,
        }
    }
}
