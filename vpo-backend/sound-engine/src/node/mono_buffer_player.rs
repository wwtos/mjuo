use crate::{util::interpolate::hermite_lookup, MonoSample, SoundConfig};

#[derive(Debug, Clone)]
pub struct MonoBufferPlayer {
    global_sample_rate: u32,
    buffer_rate: u32,
    playback_rate: f32,
    adjusted_playback_rate: f32,
    audio_position: f32,
    sample_length: usize,
}

impl MonoBufferPlayer {
    pub fn new(config: &SoundConfig, buffer: &MonoSample) -> MonoBufferPlayer {
        let buffer_rate = buffer.sample_rate;
        let sample_length = buffer.audio_raw.len();

        MonoBufferPlayer {
            playback_rate: 1.0,
            buffer_rate,
            adjusted_playback_rate: buffer_rate as f32 / config.sample_rate as f32,
            global_sample_rate: config.sample_rate,
            audio_position: 0.0,
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
}

impl MonoBufferPlayer {
    pub fn get_next_sample(&mut self, buffer: &MonoSample) -> f32 {
        let buffer_position_unsafe = self.audio_position as i64;

        if buffer_position_unsafe < 1 {
            self.audio_position += self.adjusted_playback_rate;
            return 0.0;
        }

        // if it's done playing, it'll automatically stop
        if buffer_position_unsafe > (self.sample_length as i64) - 3 {
            return 0.0; // out of interpolation bounds
        }

        let out = hermite_lookup(&buffer.audio_raw, self.audio_position);

        self.audio_position += self.adjusted_playback_rate;

        out
    }

    pub fn seek(&mut self, location: f32) {
        self.audio_position = location;
    }
}
