use crate::{MonoSample, SoundConfig};

use super::{interpolate::hermite_interpolate, sample::Sample};

pub enum State {
    Looping,
    Releasing,
}

#[derive(Debug)]
pub struct SamplePlayer {
    audio_position: f32,
    global_sample_rate: u32,
    buffer_rate: u32,
    playback_rate: f32,
    adjusted_playback_rate: f32,
    sample_length: usize,
}

impl SamplePlayer {
    pub fn new(config: &SoundConfig, buffer: &MonoSample) -> SamplePlayer {
        let buffer_rate = buffer.sample_rate;
        let sample_length = buffer.audio_raw.len();

        SamplePlayer {
            audio_position: 0.0,
            global_sample_rate: config.sample_rate,
            buffer_rate,
            playback_rate: 1.0,
            adjusted_playback_rate: buffer_rate as f32 / config.sample_rate as f32,
            sample_length,
        }
    }

    pub fn get_playback_rate(&self) -> f32 {
        self.playback_rate
    }

    pub fn set_playback_rate(&mut self, playback_rate: f32) {
        self.playback_rate = playback_rate;
        self.adjusted_playback_rate = (self.buffer_rate as f32 / self.global_sample_rate as f32) * playback_rate;
    }

    fn next_sample(&mut self, sample: &Sample) -> f32 {
        let buffer_position_unbounded = self.audio_position as i64;

        if buffer_position_unbounded < 1 {
            self.audio_position += self.adjusted_playback_rate;
            return sample.buffer.audio_raw[buffer_position_unbounded as usize];
        }

        if buffer_position_unbounded >= self.sample_length as i64 {
            return 0.0;
        }

        if buffer_position_unbounded > (self.sample_length as i64) - 3 {
            return sample.buffer.audio_raw[buffer_position_unbounded as usize]; // out of interpolation bounds
        }

        let buffer_position = (buffer_position_unbounded - 1) as usize;
        let audio = &sample.buffer.audio_raw;

        self.audio_position += self.adjusted_playback_rate;

        hermite_interpolate(
            audio[buffer_position],
            audio[buffer_position + 1],
            audio[buffer_position + 2],
            audio[buffer_position + 3],
            self.audio_position - self.audio_position.floor(),
        )
    }
}
