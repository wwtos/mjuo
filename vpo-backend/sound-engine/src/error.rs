use snafu::prelude::*;

use crate::node::{InputType, OutputType};

#[derive(Clone, Debug, PartialEq, Snafu)]
pub enum NodeError {
    #[snafu(display("Input not supported: {unsupported_input_type}"))]
    UnsupportedInput { unsupported_input_type: InputType },
    #[snafu(display("Output not supported: {unsupported_output_type}"))]
    UnsupportedOutput { unsupported_output_type: OutputType },
    #[snafu(display("Ramp out of range: (from {ramp_from} to {ramp_to})"))]
    RampOutOfRange { ramp_from: f32, ramp_to: f32 },
}
