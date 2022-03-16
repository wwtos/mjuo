use alsa::seq::Input;
use simple_error::bail;
use simple_error::SimpleError;

use crate::constants::{SAMPLE_RATE, TWO_PI};

use crate::error::NodeError;
use crate::node::{AudioNode, InputType, OutputType};
use crate::wave::interpolate::interpolate;
use crate::wave::tables::{WAVETABLE_SIZE};
use crate::wave::tables::{SAWTOOTH_VALUES, SINE_VALUES, SQUARE_VALUES, TRIANGLE_VALUES};

pub trait Oscillator {
    fn get_frequency(&self) -> f32;
    fn set_frequency(&mut self, frequency: f32);
}

pub enum Waveform {
    Sine,
    Triangle,
    Sawtooth,
    Square,
}

/// A sinsouid oscillator
///
/// # Inputs
/// None currently.
///
/// # Outputs
/// `out` - Mono waveform out.
pub struct OscillatorNode {
    phase: f32,
    frequency: f32,
    output_out: f32,
    wavetable: &'static Vec<[f32; WAVETABLE_SIZE]>,
}

impl OscillatorNode {
    pub fn new(waveform: Waveform) -> OscillatorNode {
        OscillatorNode {
            phase: 0_f32,
            frequency: 440_f32,
            output_out: 0_f32,
            wavetable: match waveform {
                Waveform::Sine => &*SINE_VALUES,
                Waveform::Square => &*SQUARE_VALUES,
                Waveform::Sawtooth => &*SAWTOOTH_VALUES,
                Waveform::Triangle => &*TRIANGLE_VALUES,
            },
        }
    }

    pub fn new_with_frequency(waveform: Waveform, frequency: f32) -> OscillatorNode {
        let mut oscillator = OscillatorNode::new(waveform);
        oscillator.set_frequency(frequency);

        oscillator
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.wavetable = match waveform {
            Waveform::Sine => &*SINE_VALUES,
            Waveform::Square => &*SQUARE_VALUES,
            Waveform::Sawtooth => &*SAWTOOTH_VALUES,
            Waveform::Triangle => &*TRIANGLE_VALUES,
        };
    }
}

impl Oscillator for OscillatorNode {
    fn get_frequency(&self) -> f32 {
        self.frequency
    }

    fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
}

impl AudioNode for OscillatorNode {
    fn process(&mut self) {
        let phase_advance = self.frequency / (SAMPLE_RATE as f32) * TWO_PI;
        self.phase = (self.phase + phase_advance) % TWO_PI;

        self.output_out = interpolate(self.wavetable, self.frequency, self.phase);
    }

    fn receive_audio(&mut self, input_type: InputType, _input: f32) -> Result<(), NodeError> {
        Err(NodeError::UnsupportedInput { unsupported_input_type: input_type })
    }

    fn get_output_audio(&self, output_type: OutputType) -> Result<f32, NodeError> {
        match output_type {
            OutputType::Out => Ok(self.output_out),
            _ => Err(NodeError::UnsupportedOutput { unsupported_output_type: output_type }),
        }
    }

    fn list_inputs(&self) -> Vec<InputType> {
        vec![]
    }

    fn list_outputs(&self) -> Vec<OutputType> {
        vec![OutputType::Out]
    }
}
