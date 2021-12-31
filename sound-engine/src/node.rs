pub mod envelope;

use crate::error::NodeError;

pub trait AudioNode {
    fn process(&mut self);
    fn receive_audio(&mut self, input_type: InputType, input: f32) -> Result<(), NodeError>;
    fn get_output_audio(&self, output_type: OutputType) -> Result<f32, NodeError>;
    fn list_inputs(&self) -> Vec<InputType>;
    fn list_outputs(&self) -> Vec<OutputType>;
}

#[derive(Debug)]
pub enum InputType {
    In,
    Gate,
    Detune,
    FilterOffset,
    Dynamic(u64),
}

#[derive(Debug)]
pub enum OutputType {
    Out,
    Gate,
    Dynamic(u64),
}
