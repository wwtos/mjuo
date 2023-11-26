use snafu::prelude::*;

#[derive(Clone, Debug, PartialEq, Snafu)]
pub enum NodeError {
    #[snafu(display("Ramp out of range: (from {ramp_from} to {ramp_to})"))]
    RampOutOfRange { ramp_from: f32, ramp_to: f32 },
}
