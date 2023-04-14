use crate::{midi::messages::Note, MonoSample};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Pipe {
    pub loop_start: usize,
    pub loop_end: usize,
    pub decay_index: usize,
    pub sustain_index: usize,
    pub release_index: usize,
    pub min_release_length: usize,
    pub crossfade: usize,
    pub crossfade_release: usize,
    pub note: Note,
    pub cents: i16,
    #[serde(skip)]
    pub buffer: MonoSample,
}

impl Default for Pipe {
    fn default() -> Self {
        Self {
            loop_start: 0,
            loop_end: 100,
            decay_index: 50,
            sustain_index: 80,
            release_index: 100,
            min_release_length: 5000,
            crossfade: 256,
            crossfade_release: 256,
            note: 69,
            cents: 0,
            buffer: MonoSample::default(),
        }
    }
}
