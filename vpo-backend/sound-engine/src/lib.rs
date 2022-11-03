use std::{fs::File, io::BufReader, path::Path};

use asset_manager::{Asset, IOSnafu, LoadingError};
use rodio::{Decoder, Source};
use snafu::ResultExt;
use std;

pub mod backend;
pub mod error;
pub mod midi;
pub mod node;
pub mod openal;
pub mod ringbuffer;
pub mod sampling;
pub mod util;
pub mod wave;

pub type SamplePoint = i16;

#[derive(Debug, Clone)]
pub struct SoundConfig {
    pub sample_rate: u32,
}

pub struct MonoSample {
    pub audio_raw: Vec<f32>,
    pub sample_rate: u32,
}

impl Default for MonoSample {
    fn default() -> Self {
        MonoSample {
            sample_rate: 44_100,
            audio_raw: Vec::new(),
        }
    }
}

impl Asset for MonoSample {
    fn load_asset(path: &Path) -> Result<Self, LoadingError>
    where
        Self: Sized,
    {
        let file = BufReader::new(File::open(path).context(IOSnafu)?);
        let source = Decoder::new(file).unwrap();

        let sample_rate = source.sample_rate();
        let buffer: Vec<f32> = source.map(|x| x as f32 / i16::MAX as f32).collect();

        Ok(MonoSample {
            audio_raw: buffer,
            sample_rate,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

pub mod constants {
    #[allow(clippy::excessive_precision)]
    pub const PI: f32 = 3.14159265358979323846264338327950288f32;
    pub const TWO_PI: f32 = PI * 2.0;
    pub const BUFFER_SIZE: usize = 256;
    pub const SAMPLE_RATE: u32 = 48_000;
}
