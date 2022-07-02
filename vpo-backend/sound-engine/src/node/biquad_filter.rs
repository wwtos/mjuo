use std::f32::consts::PI;

use serde::{Serialize, Deserialize};

use crate::error::NodeError;
use crate::node::{AudioNode, InputType, OutputType};
use crate::SoundConfig;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum BiquadFilterType {
    Lowpass,
    Highpass,
    Bandpass,
    Notch,
    Allpass,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BiquadFilter {
    filter_type: BiquadFilterType,
    sample_rate: u32,
    frequency: f32,
    q: f32,
    dirty: bool,
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    prev_input_1: f32,
    prev_input_2: f32,
    prev_output_1: f32,
    prev_output_2: f32,
    input_in: f32,
    output_out: f32,
}

impl BiquadFilter {
    pub fn new(config: &SoundConfig, filter_type: BiquadFilterType, frequency: f32, q: f32) -> BiquadFilter {
        let mut new_filter = BiquadFilter {
            filter_type,
            sample_rate: config.sample_rate,
            frequency,
            q,
            a1: 0.0,
            a2: 0.0,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            prev_input_1: 0.0,
            prev_input_2: 0.0,
            prev_output_1: 0.0,
            prev_output_2: 0.0,
            input_in: 0_f32,
            output_out: 0_f32,
            dirty: true,
        };

        new_filter.recompute();

        new_filter
    }

    pub fn filter_audio(&mut self, input_in: f32) -> f32 {
        if self.dirty {
            self.recompute();
        }

        let output =
            (self.b0 * input_in) + (self.b1 * self.prev_input_1) + (self.b2 * self.prev_input_2)
                - (self.a1 * self.prev_output_1)
                - (self.a2 * self.prev_output_2);

        self.prev_input_2 = self.prev_input_1;
        self.prev_input_1 = input_in;

        self.prev_output_2 = self.prev_output_1;
        self.prev_output_1 = output;

        output
    }

    pub fn process(&mut self) {
        self.output_out = self.filter_audio(self.input_in);
    }

    pub fn set_params(&mut self, frequency: f32, q: f32) {
        self.frequency = frequency.clamp(0.01, self.sample_rate as f32 * 0.5);
        self.q = q;
        self.dirty = true;
    }

    pub fn recompute(&mut self) {
        let a1;
        let a2;
        let b0;
        let b1;
        let b2;

        // thanks to JUCE modules/juce_audio_basics/effects/juce_IIRFilter.cpp
        match &self.filter_type {
            BiquadFilterType::Lowpass => {
                let n = 1.0 / f32::tan(PI * self.frequency / self.sample_rate as f32);
                let n_squared = n * n;
                let c1 = 1.0 / (1.0 + 1.0 / self.q * n + n_squared);

                b0 = c1;
                b1 = c1 * 2.0;
                b2 = c1;
                a1 = c1 * 2.0 * (1.0 - n_squared);
                a2 = c1 * (1.0 - 1.0 / self.q * n + n_squared);
            }
            BiquadFilterType::Highpass => {
                let n = 1.0 / f32::tan(PI * self.frequency / self.sample_rate as f32);
                let n_squared = n * n;
                let c1 = 1.0 / (1.0 + 1.0 / self.q * n + n_squared);

                b0 = c1;
                b1 = c1 * -2.0;
                b2 = c1;
                a1 = c1 * 2.0 * (n_squared - 1.0);
                a2 = c1 * (1.0 - 1.0 / self.q * n + n_squared);
            }
            BiquadFilterType::Bandpass => {
                let n = 1.0 / f32::tan(PI * self.frequency / self.sample_rate as f32);
                let n_squared = n * n;
                let c1 = 1.0 / (1.0 + 1.0 / self.q * n + n_squared);

                b0 = c1 * n / self.q;
                b1 = 0.0;
                b2 = -c1 * n / self.q;
                a1 = c1 * 2.0 * (1.0 - n_squared);
                a2 = c1 * (1.0 - 1.0 / self.q * n + n_squared);
            }
            BiquadFilterType::Notch => {
                let n = 1.0 / f32::tan(PI * self.frequency / self.sample_rate as f32);
                let n_squared = n * n;
                let c1 = 1.0 / (1.0 + 1.0 / self.q * n + n_squared);

                b0 = c1 * (1.0 + n_squared);
                b1 = 2.0 * c1 * (1.0 - n_squared);
                b2 = c1 * (1.0 + n_squared);
                a1 = c1 * 2.0 * (1.0 - n_squared);
                a2 = c1 * (1.0 - n / self.q + n_squared);
            }
            BiquadFilterType::Allpass => {
                let n = 1.0 / f32::tan(PI * self.frequency / self.sample_rate as f32);
                let n_squared = n * n;
                let c1 = 1.0 / (1.0 + 1.0 / self.q * n + n_squared);

                b0 = c1 * (1.0 - n / self.q + n_squared);
                b1 = c1 * 2.0 * (1.0 - n_squared);
                b2 = 1.0;
                a1 = c1 * 2.0 * (1.0 - n_squared);
                a2 = c1 * (1.0 - n / self.q + n_squared);
            }
        };

        self.a1 = a1;
        self.a2 = a2;
        self.b0 = b0;
        self.b1 = b1;
        self.b2 = b2;

        self.dirty = false;
    }

    pub fn get_filter_type(&self) -> BiquadFilterType {
        self.filter_type
    }
    
    pub fn set_filter_type(&mut self, filter_type: BiquadFilterType) {
        self.dirty = true;
        self.filter_type = filter_type;
    }

    pub fn get_frequency(&self) -> f32 {
        self.frequency
    }
    
    pub fn set_frequency(&mut self, frequency: f32) {
        self.dirty = true;
        self.frequency = frequency;
    }

    pub fn get_q(&self) -> f32 {
        self.q
    }
    
    pub fn set_q(&mut self, q: f32) {
        self.dirty = true;
        self.q = q;
    }

    pub fn set_audio_in(&mut self, audio: f32) {
        self.input_in = audio;
    }

    pub fn get_output_out(&self) -> f32 {
        self.output_out
    }
}

