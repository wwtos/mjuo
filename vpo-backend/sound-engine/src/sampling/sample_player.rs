use std::thread;

use super::audio_loader::{self, AudioLoader};

pub enum SampleMessage {
    Exit,
}

pub struct SamplePlayer<'a> {
    audio_loader: &'a AudioLoader,
}

impl SamplePlayer<'_> {
    pub fn new(audio_loader: &AudioLoader) -> SamplePlayer {
        SamplePlayer {
            audio_loader: audio_loader,
        }
    }
}
