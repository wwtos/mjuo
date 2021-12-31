pub mod envelope;

use simple_error::SimpleError;

pub trait AudioNode {
    fn process(&mut self);
    fn receive_audio(&mut self, input_type: InputType, input: f32) -> Result<(), SimpleError>;
    fn get_output_audio(&self, output_type: OutputType) -> Result<f32, SimpleError>;
}

#[derive(Debug)]
pub enum InputType {
    In,
    Gate,
    Detune,
    FilterOffset,
    Dynamic(u64)
}

#[derive(Debug)]
pub enum OutputType {
    Out,
    Gate,
    Dynamic(u64)
}

