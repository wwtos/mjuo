#[cfg(target_os = "linux")]
pub mod alsa;
#[cfg(target_os = "linux")]
pub mod alsa_midi;
pub mod midir;
#[cfg(target_os = "linux")]
pub mod pulse;

pub mod cpal;

use std::error::Error;

use crate::constants::BUFFER_SIZE;

pub trait AudioClientBackend {
    fn write(&mut self, data: &[f32; BUFFER_SIZE]) -> Result<(), Box<dyn Error>>;
    fn connect(&mut self) -> Result<(), Box<dyn Error>>;
    fn drain(&self) -> Result<(), Box<dyn Error>>;
}

pub trait MidiClientBackend {
    fn read(&self) -> Result<Vec<u8>, Box<dyn Error>>;
    fn connect(&mut self) -> Result<(), Box<dyn Error>>;
}
