use crate::{error::NodeError, SoundConfig};

#[derive(Debug, Clone)]
pub enum RampType {
    Linear,
    Exponential,
}

/// note: exponential ramp cannot use a negative value for from or to!
#[derive(Debug, Clone)]
pub struct Ramp {
    sample_rate: u32,
    from: f32,
    to: f32,
    at: f32,
    speed: f32,
    duration: f32,
    from_processed: f32, // processed meaning whatever form the ramp type needs the values in for fast calculation
    to_processed: f32,
    ramp_type: RampType,
}

impl Ramp {
    pub fn new(sound_config: &SoundConfig) -> Ramp {
        Ramp {
            sample_rate: sound_config.sample_rate,
            from: 0.0,
            to: 0.0,
            at: 0.0,
            speed: 0.0,
            duration: 0.0,
            from_processed: 0.0,
            to_processed: 0.0,
            ramp_type: RampType::Linear,
        }
    }

    pub fn new_with_start_value(sound_config: &SoundConfig, start: f32) -> Ramp {
        Ramp {
            sample_rate: sound_config.sample_rate,
            from: start,
            to: start,
            at: start,
            speed: 0.0,
            duration: 0.0,
            from_processed: start,
            to_processed: start,
            ramp_type: RampType::Linear,
        }
    }

    pub fn process(&mut self) -> f32 {
        self.at += self.speed;
        self.at = self.at.clamp(
            f32::min(self.from_processed, self.to_processed),
            f32::max(self.from_processed, self.to_processed),
        );

        self.get_position()
    }

    /// duration is in seconds
    pub fn set_ramp_parameters(&mut self, from: f32, to: f32, duration: f32) -> Result<(), NodeError> {
        self.from = from;
        self.to = to;
        self.duration = duration;

        match self.ramp_type {
            RampType::Linear => {
                self.from_processed = self.from;
                self.to_processed = self.to;

                self.at = self.from;
                self.speed = ((self.to - self.from) / self.duration) / self.sample_rate as f32;
            }
            RampType::Exponential => {
                if self.from < 0.0 || self.to < 0.0 {
                    return Err(NodeError::RampOutOfRange {
                        ramp_from: self.from,
                        ramp_to: self.to,
                    });
                    // that is, unless my imaginary number knowledge was better
                }

                // exponential formula in this case is
                // ramp_value = from * 2^at
                // the values that need to be calculated are
                // how far to go linearly to end up in the from-to range exponentially

                self.from_processed = 0.0;
                self.at = 0.0;

                // from * 2^x = to (maximum position)
                // solved is
                // x = log2(to/from)
                // where x is how far to go on the exponential curve before stopping
                if (self.to - self.from).abs() < f32::EPSILON {
                    self.to_processed = 0.0;
                } else {
                    self.to_processed = f32::ln(self.to / self.from) / f32::ln(2.0);
                }

                self.speed = (self.to_processed / self.duration) / self.sample_rate as f32;
            }
        }

        Ok(())
    }

    pub fn set_position(&mut self, value: f32) {
        self.ramp_type = RampType::Linear;

        self.from = value;
        self.to = value;
        self.from_processed = value;
        self.to_processed = value;
        self.at = value;

        self.duration = 0.0;
    }

    pub fn get_position(&self) -> f32 {
        match self.ramp_type {
            RampType::Linear => self.at,
            RampType::Exponential => {
                (self.from * 2_f32.powf(self.at)).clamp(f32::min(self.from, self.to), f32::max(self.from, self.to))
            }
        }
    }

    pub fn is_done(&self) -> bool {
        (self.to - self.get_position()).abs() < 0.001
    }

    pub fn get_to(&self) -> f32 {
        self.to
    }

    pub fn set_ramp_type(&mut self, ramp_type: RampType) -> Result<(), NodeError> {
        let from = self.get_position();

        self.ramp_type = ramp_type;

        self.set_ramp_parameters(from, self.to, self.duration)
    }

    pub fn ramp_to_value(&mut self, to: f32, duration: f32) -> Result<(), NodeError> {
        self.set_ramp_parameters(self.get_position(), to, duration)
    }
}
