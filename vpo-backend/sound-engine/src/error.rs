use std::fmt;
use snafu::prelude::*;

use crate::node::{InputType, OutputType};

#[derive(Clone, Debug, PartialEq, Snafu)]
pub enum NodeError {
    #[snafu(display("Input not supported: {unsupported_input_type}"))]
    UnsupportedInput { unsupported_input_type: InputType },
    #[snafu(display("Output not supported: {unsupported_output_type}"))]
    UnsupportedOutput { unsupported_output_type: OutputType }
}
