pub mod alsa_midi;
pub mod pulse;

use std::error::Error;

pub trait AudioClientBackend {
    fn write(&self, data: &[f32]) -> Result<(), Box<dyn Error>>;
    fn connect(&mut self) -> Result<(), Box<dyn Error>>;
    fn drain(&self) -> Result<(), Box<dyn Error>>;
}

pub trait MidiClientBackend {
    fn read(&self) -> Result<Vec<u8>, Box<dyn Error>>;
    fn connect(&mut self) -> Result<(), Box<dyn Error>>;
}
