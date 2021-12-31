use crate::node::{AudioNode, InputType, OutputType};
use crate::{error::NodeError, error::NodeErrorType};

pub struct Dummy {
    input_in: f32,
    output_out: f32,
}

impl Dummy {
    pub fn new() -> Dummy {
        Dummy {
            input_in: 0_f32,
            output_out: 0_f32,
        }
    }

    pub fn set_output_out(&mut self, output_out: f32) {
        self.output_out = output_out;
    }

    pub fn get_input_in(&self) -> f32 {
        self.input_in
    }
}

impl Dummy {
    pub fn receive_audio(&mut self, input: f32) {
        self.input_in = input;
    }

    pub fn get_output_audio(&self) -> f32 {
        self.output_out
    }
}

impl Default for Dummy {
    fn default() -> Self {
        Self::new()
    }
}


impl AudioNode for Dummy {
    fn process(&mut self) {
        
    }

    fn receive_audio(&mut self, input_type: InputType, input: f32) -> Result<(), NodeError> {
        match input_type {
            InputType::In => {
                self.input_in = input;

                Ok(())
            }
            _ => Err(NodeError::new(
                format!("Envelope cannot input audio of type {:?}", input_type),
                NodeErrorType::UnsupportedInput,
            )),
        }
    }

    fn get_output_audio(&self, output_type: OutputType) -> Result<f32, NodeError> {
        match output_type {
            OutputType::Out => Ok(self.output_out),
            _ => Err(NodeError::new(
                format!("Envelope cannot output audio of type {:?}", output_type),
                NodeErrorType::UnsupportedOutput,
            )),
        }
    }

    fn list_inputs(&self) -> Vec<InputType> {
        vec![InputType::In]
    }

    fn list_outputs(&self) -> Vec<OutputType> {
        vec![OutputType::Out]
    }
}

