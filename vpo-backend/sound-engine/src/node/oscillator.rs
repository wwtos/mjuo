use serde::Deserialize;
use serde::Serialize;

use crate::constants::{SAMPLE_RATE, TWO_PI};

use crate::error::NodeError;
use crate::node::{AudioNode, InputType, OutputType};
use crate::wave::interpolate::interpolate;
use crate::wave::tables::WAVETABLE_SIZE;
use crate::wave::tables::{SAWTOOTH_VALUES, SINE_VALUES, SQUARE_VALUES, TRIANGLE_VALUES};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Waveform {
    Sine,
    Triangle,
    Sawtooth,
    Square,
}

impl Waveform {
    pub fn from_string(waveform: &str) -> Option<Waveform> {
        match waveform {
            "sine" => Some(Waveform::Sine),
            "triangle" => Some(Waveform::Triangle),
            "sawtooth" => Some(Waveform::Sawtooth),
            "square" => Some(Waveform::Square),
            _ => None,
        }
    }
}

pub fn wavetable_lookup(waveform: &Waveform) -> &'static Vec<[f32; WAVETABLE_SIZE]> {
    match waveform {
        Waveform::Sine => &SINE_VALUES,
        Waveform::Triangle => &TRIANGLE_VALUES,
        Waveform::Sawtooth => &SAWTOOTH_VALUES,
        Waveform::Square => &SQUARE_VALUES,
    }
}

/// A sinsouid oscillator
///
/// # Inputs
/// None currently.
///
/// # Outputs
/// `out` - Mono waveform out.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Oscillator {
    phase: f32,
    frequency: f32,
    output_out: f32,
    waveform: Waveform,
}

impl Oscillator {
    pub fn new(waveform: Waveform) -> Oscillator {
        Oscillator {
            phase: 0_f32,
            frequency: 440_f32,
            output_out: 0_f32,
            waveform,
        }
    }

    pub fn new_with_frequency(waveform: Waveform, frequency: f32) -> Oscillator {
        let mut oscillator = Oscillator::new(waveform);
        oscillator.set_frequency(frequency);

        oscillator
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    pub fn get_phase(&self) -> f32 {
        self.phase
    }

    pub fn set_phase(&mut self, phase: f32) {
        self.phase = phase;
    }

    #[inline]
    pub fn process_fast(&mut self) -> f32 {
        let phase_advance = self.frequency / (SAMPLE_RATE as f32) * TWO_PI;
        self.phase = (self.phase + phase_advance) % TWO_PI;

        interpolate(wavetable_lookup(&self.waveform), self.frequency, self.phase)
    }
}

impl Oscillator {
    pub fn get_frequency(&self) -> f32 {
        self.frequency
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
}

impl AudioNode for Oscillator {
    fn process(&mut self) {
        self.output_out = self.process_fast();
    }

    fn receive_audio(&mut self, input_type: InputType, _input: f32) -> Result<(), NodeError> {
        Err(NodeError::UnsupportedInput {
            unsupported_input_type: input_type,
        })
    }

    fn get_output_audio(&self, output_type: OutputType) -> Result<f32, NodeError> {
        match output_type {
            OutputType::Out => Ok(self.output_out),
            _ => Err(NodeError::UnsupportedOutput {
                unsupported_output_type: output_type,
            }),
        }
    }

    fn list_inputs(&self) -> Vec<InputType> {
        vec![]
    }

    fn list_outputs(&self) -> Vec<OutputType> {
        vec![OutputType::Out]
    }
}
