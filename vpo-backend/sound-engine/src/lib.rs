#![allow(confusable_idents)]

use std::fmt::Debug;

use midi::messages::MidiMessage;
use serde::Serialize;
use smallvec::SmallVec;

pub mod error;
pub mod midi;
pub mod node;
pub mod openal;
pub mod ringbuffer;
pub mod sampling;
pub mod util;
pub mod wave;

pub type SamplePoint = i16;
pub type MidiBundle = SmallVec<[MidiMessage; 2]>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundConfig {
    pub sample_rate: u32,
    pub buffer_size: usize,
}

impl Default for SoundConfig {
    fn default() -> Self {
        SoundConfig {
            sample_rate: 200,
            buffer_size: 1,
        }
    }
}

pub struct MonoSample {
    pub audio_raw: Vec<f32>,
    pub sample_rate: u32,
}

impl Debug for MonoSample {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Mono audio sample, {:.2}s]",
            self.audio_raw.len() as f32 / self.sample_rate as f32
        )
    }
}

impl Default for MonoSample {
    fn default() -> Self {
        MonoSample {
            sample_rate: 44_100,
            audio_raw: Vec::new(),
        }
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
