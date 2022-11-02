use crate::{MonoSample, SoundConfig};

#[derive(Debug)]
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
            buffer_rate: buffer_rate,
            adjusted_playback_rate: buffer_rate as f32 / config.sample_rate as f32,
            global_sample_rate: config.sample_rate,
            audio_position: 0.0,
            sample_length,
        }
    }

    pub fn get_playback_rate(&self) -> f32 {
        self.adjusted_playback_rate
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

        let buffer_position = (buffer_position_unsafe - 1) as usize;
        let sample = &buffer.audio_raw;

        self.audio_position += self.adjusted_playback_rate;

        hermite_interpolate(
            sample[buffer_position],
            sample[buffer_position + 1],
            sample[buffer_position + 2],
            sample[buffer_position + 3],
            self.audio_position % 1.0,
        )
    }

    pub fn seek(&mut self, location: f32) {
        self.audio_position = location;
    }
}

// (elephant paper) http://yehar.com/blog/wp-content/uploads/2009/08/deip.pdf
// https://stackoverflow.com/questions/1125666/how-do-you-do-bicubic-or-other-non-linear-interpolation-of-re-sampled-audio-da
fn hermite_interpolate(x0: f32, x1: f32, x2: f32, x3: f32, t: f32) -> f32 {
    let c0 = x1;
    let c1 = 0.5 * (x2 - x0);
    let c2 = x0 - (2.5 * x1) + (2.0 * x2) - (0.5 * x3);
    let c3 = (0.5 * (x3 - x0)) + (1.5 * (x1 - x2));

    (((((c3 * t) + c2) * t) + c1) * t) + c0
}
