pub mod biquad_filter;
pub mod dummy;
pub mod envelope;
pub mod mono_buffer_player;
pub mod oscillator;

use core::fmt;

use crate::error::NodeError;
use crate::midi::messages::MidiData;
use simple_error::SimpleError;

pub trait AudioNode {
    fn process(&mut self);
    fn receive_audio(&mut self, input_type: InputType, input: f32) -> Result<(), NodeError>;
    fn get_output_audio(&self, output_type: OutputType) -> Result<f32, NodeError>;
    fn list_inputs(&self) -> Vec<InputType>;
    fn list_outputs(&self) -> Vec<OutputType>;
}

pub trait MidiNode {
    fn receive_midi(&mut self, input: &[MidiData]) -> Result<(), SimpleError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputType {
    In,
    Gate,
    Detune,
    FilterOffset,
    Dynamic(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputType {
    Out,
    Gate,
    Dynamic(u64),
}

impl fmt::Display for InputType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for OutputType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
