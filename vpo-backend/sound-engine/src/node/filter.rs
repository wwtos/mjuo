use crate::constants::PI;
use crate::node::{AudioNode, InputType, OutputType};
use crate::SoundConfig;
use crate::{error::NodeError, error::NodeErrorType};

#[derive(Clone, Copy)]
pub enum FilterType {
    Lowpass,
}

pub struct Filter {
    filter_type: FilterType,
    global_sample_rate: u32,
    frequency: f32,
    q: f32,
    dirty: bool,
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    prev_offset: f32,
    prev_input_1: f32,
    prev_input_2: f32,
    prev_output_1: f32,
    prev_output_2: f32,
    filter_offset_in: f32,
    input_in: f32,
    output_out: f32,
}

impl Filter {
    pub fn new(config: SoundConfig, filter_type: FilterType, frequency: f32, q: f32) -> Filter {
        let mut new_filter = Filter {
            filter_type,
            global_sample_rate: config.sample_rate,
            frequency,
            q,
            a1: 0.0,
            a2: 0.0,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            prev_offset: 0.0,
            prev_input_1: 0.0,
            prev_input_2: 0.0,
            prev_output_1: 0.0,
            prev_output_2: 0.0,
            filter_offset_in: 0.0,
            input_in: 0_f32,
            output_out: 0_f32,
            dirty: true,
        };

        new_filter.recompute();

        new_filter
    }

    fn filter_audio(&mut self, input_in: f32, filter_offset_in: f32) -> f32 {
        if f32::abs(filter_offset_in - self.prev_offset) > f32::EPSILON || self.dirty {
            // avoid excess recomputation
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

        self.prev_offset = filter_offset_in;

        output
    }
}

impl Filter {
    fn recompute(&mut self) {
        let a1;
        let a2;
        let b0;
        let b1;
        let b2;

        match &self.filter_type {
            FilterType::Lowpass => {
                let freq = (self.frequency * f32::powf(2.0, self.filter_offset_in))
                    .clamp(0.01, self.global_sample_rate as f32 * 0.5);
                //println!("{}", freq);

                let k = (PI * freq / self.global_sample_rate as f32).tan();
                let norm = 1.0 / (1.0 + k / self.q + k * k);

                b0 = k * k * norm;
                b1 = 2.0 * b0;
                b2 = b0;
                a1 = 2.0 * (k * k - 1.0) * norm;
                a2 = (1.0 - k / self.q + k * k) * norm;
            }
        };

        self.a1 = a1;
        self.a2 = a2;
        self.b0 = b0;
        self.b1 = b1;
        self.b2 = b2;

        self.dirty = false;
    }

    pub fn get_filter_type(&self) -> FilterType {
        self.filter_type
    }
    pub fn set_filter_type(&mut self, filter_type: FilterType) {
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
}

impl AudioNode for Filter {
    fn process(&mut self) {
        self.output_out = self.filter_audio(self.input_in, self.filter_offset_in);
    }

    fn receive_audio(&mut self, input_type: InputType, input: f32) -> Result<(), NodeError> {
        match input_type {
            InputType::In => {
                self.input_in = input;

                Ok(())
            }
            InputType::Detune => {
                self.filter_offset_in = input;

                Ok(())
            }
            _ => Err(NodeError::new(
                format!("Filter cannot input audio of type {:?}", input_type),
                NodeErrorType::UnsupportedInput,
            )),
        }
    }

    fn get_output_audio(&self, output_type: OutputType) -> Result<f32, NodeError> {
        match output_type {
            OutputType::Out => Ok(self.output_out),
            _ => Err(NodeError::new(
                format!("Filter cannot output audio of type {:?}", output_type),
                NodeErrorType::UnsupportedOutput,
            )),
        }
    }

    fn list_inputs(&self) -> Vec<InputType> {
        vec![InputType::In, InputType::Detune]
    }

    fn list_outputs(&self) -> Vec<OutputType> {
        vec![OutputType::Out]
    }
}
