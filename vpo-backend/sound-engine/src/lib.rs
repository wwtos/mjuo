#![allow(confusable_idents)]

use std::fmt::Debug;

use clocked::midi::MidiMessage;
use common::alloc::Alloc;
use serde::Serialize;

pub mod error;
pub mod node;
pub mod openal;
pub mod ringbuffer;
pub mod sampling;
pub mod util;
pub mod wave;

pub type SamplePoint = i16;

pub type MidiChannel = Vec<MidiMessage>;
pub struct MidiIndex<'a>(Alloc<'a, MidiChannel>);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundConfig {
    pub sample_rate: u32,
    pub buffer_size: usize,
}

impl Default for SoundConfig {
    fn default() -> Self {
        SoundConfig {
            sample_rate: 44_100,
            buffer_size: 64,
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
