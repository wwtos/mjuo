use crate::{MonoSample, SoundConfig};

use super::{
    interpolate::{hermite_interpolate, lerp},
    sample::Sample,
    util::{rms32, sq32},
};

#[derive(Debug, Clone)]
enum State {
    Attacking,
    Looping,
    AboutToRelease,
    Releasing,
    ReleasingAfterAttack,
}

#[derive(Debug, Clone)]
pub struct SamplePlayer {
    state: State,
    audio_position: f64,
    audio_position_release: f64,
    stop_at_index: usize,
    release_search_width: usize,
    release_amplitude: f32,
    release_length: f64,
    global_sample_rate: u32,
    buffer_rate: u32,
    playback_rate: f64,
    adjusted_playback_rate: f64,
    sample_length: usize,
}

impl SamplePlayer {
    pub fn new(config: &SoundConfig, sample: &Sample) -> SamplePlayer {
        let buffer_rate = sample.buffer.sample_rate;
        let sample_length = sample.buffer.audio_raw.len();

        // look for potential release locations based on frequency
        let freq = (440.0 / 32.0) * 2_f32.powf((sample.note - 9) as f32 / 12.0);
        let release_search_width = (buffer_rate as f32 / freq) as usize * 2;

        SamplePlayer {
            state: State::Attacking,
            audio_position: 0.0,
            audio_position_release: 0.0,
            stop_at_index: 0,
            release_search_width,
            release_amplitude: 1.0,
            release_length: 0.0,
            global_sample_rate: config.sample_rate,
            buffer_rate,
            playback_rate: 1.0,
            adjusted_playback_rate: buffer_rate as f64 / config.sample_rate as f64,
            sample_length,
        }
    }

    pub fn get_playback_rate(&self) -> f64 {
        self.playback_rate
    }

    pub fn set_playback_rate(&mut self, playback_rate: f64) {
        self.playback_rate = playback_rate;
        self.adjusted_playback_rate = (self.buffer_rate as f64 / self.global_sample_rate as f64) * playback_rate;
    }

    pub fn seek(&mut self, location: usize) {
        self.audio_position = location as f64;
        self.state = State::Attacking;
    }

    #[inline]
    fn next_sample_looped(&mut self, sample: &Sample, loop_start: usize, loop_end: usize) -> f32 {
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

        let mut buffer_position = (buffer_position_unbounded - 1) as usize;

        self.audio_position += self.adjusted_playback_rate;

        if sample.crossfade > 3 {
            if buffer_position >= loop_end + sample.crossfade {
                buffer_position = loop_start + sample.crossfade;
                self.audio_position = (buffer_position + 1) as f64;
            }

            hermite_interpolate(
                audio_lookup_with_crossfade(
                    buffer_position,
                    loop_start,
                    loop_end,
                    &sample.buffer,
                    &sample.crossfade_buffer,
                ),
                audio_lookup_with_crossfade(
                    buffer_position + 1,
                    loop_start,
                    loop_end,
                    &sample.buffer,
                    &sample.crossfade_buffer,
                ),
                audio_lookup_with_crossfade(
                    buffer_position + 2,
                    loop_start,
                    loop_end,
                    &sample.buffer,
                    &sample.crossfade_buffer,
                ),
                audio_lookup_with_crossfade(
                    buffer_position + 3,
                    loop_start,
                    loop_end,
                    &sample.buffer,
                    &sample.crossfade_buffer,
                ),
                (self.audio_position - self.audio_position.floor()) as f32,
            )
        } else {
            hermite_interpolate(
                audio_lookup_with_loop(buffer_position, loop_start, loop_end, &sample.buffer),
                audio_lookup_with_loop(buffer_position + 1, loop_start, loop_end, &sample.buffer),
                audio_lookup_with_loop(buffer_position + 2, loop_start, loop_end, &sample.buffer),
                audio_lookup_with_loop(buffer_position + 3, loop_start, loop_end, &sample.buffer),
                (self.audio_position - self.audio_position.floor()) as f32,
            )
        }
    }

    #[inline]
    fn next_sample_released(&mut self, sample: &Sample) -> f32 {
        let buffer_position_unbounded = self.audio_position_release as i64;

        if buffer_position_unbounded >= self.sample_length as i64 {
            return 0.0;
        }

        if buffer_position_unbounded > (self.sample_length as i64) - 3 {
            return sample.buffer.audio_raw[buffer_position_unbounded as usize]; // out of interpolation bounds
        }

        let buffer_position = (buffer_position_unbounded - 1) as usize;
        let audio = &sample.buffer.audio_raw;

        self.audio_position_release += self.adjusted_playback_rate;

        hermite_interpolate(
            audio[buffer_position],
            audio[buffer_position + 1],
            audio[buffer_position + 2],
            audio[buffer_position + 3],
            (self.audio_position_release - self.audio_position_release.floor()) as f32,
        )
    }

    fn next_sample_normal(&mut self, sample: &Sample) -> f32 {
        let buffer_position_unbounded = self.audio_position as i64;

        if buffer_position_unbounded < 1 {
            self.audio_position += self.adjusted_playback_rate;
            return 0.0;
        }

        // if it's done playing, it'll automatically stop
        if buffer_position_unbounded > (self.sample_length as i64) - 3 {
            return 0.0; // out of interpolation bounds
        }

        let buffer_position = (buffer_position_unbounded - 1) as usize;
        let sample = &sample.buffer.audio_raw;

        self.audio_position += self.adjusted_playback_rate;

        hermite_interpolate(
            sample[buffer_position],
            sample[buffer_position + 1],
            sample[buffer_position + 2],
            sample[buffer_position + 3],
            (self.audio_position - self.audio_position.floor()) as f32,
        )
    }

    pub fn release(&mut self, sample: &Sample) {
        if self.audio_position < 1.0 {
            return;
        }

        let released_at = self.audio_position as usize;
        let release_index = sample.release_index;
        let audio = &sample.buffer.audio_raw;

        if released_at < 6 {
            return;
        }

        if matches!(self.state, State::Attacking) {
            self.state = State::ReleasingAfterAttack;
            self.release_length = self.audio_position;

            return;
        }

        let rms_before = rms32(&audio[released_at..(released_at + self.release_search_width)]);
        let rms_release = rms32(&audio[release_index..(release_index + self.release_search_width)]);

        self.release_amplitude = rms_before / rms_release;

        let mut lowest_score = f32::INFINITY;
        let mut stop_at_index = 0;

        for from_index in released_at..(released_at + self.release_search_width) {
            let cross = (0..10).fold(0.0, |mean, i| {
                sq32(audio[from_index + i - 5] - audio[release_index + i - 5] * self.release_amplitude) + mean
            }) / audio.len() as f32;

            if cross < lowest_score {
                stop_at_index = from_index;
                lowest_score = cross;
            }
        }

        self.stop_at_index = stop_at_index;

        self.state = State::AboutToRelease;
    }

    pub fn next_sample(&mut self, sample: &Sample) -> f32 {
        match self.state {
            State::Attacking => {
                if self.audio_position > sample.attack_index as f64 {
                    self.state = State::Looping;
                }

                self.next_sample_looped(sample, sample.loop_start, sample.loop_end)
            }
            State::Looping => self.next_sample_looped(sample, sample.loop_start, sample.loop_end),
            State::AboutToRelease => {
                if self.audio_position as usize >= self.stop_at_index {
                    self.state = State::Releasing;
                    self.audio_position_release = sample.release_index as f64;
                }

                self.next_sample_normal(sample)
            }
            State::Releasing => {
                if sample.crossfade_release > 3 {
                    let crossfade_pos = self.audio_position_release - sample.release_index as f64;

                    if (crossfade_pos as usize) < sample.crossfade_release {
                        let looped = self.next_sample_looped(sample, sample.loop_start, sample.loop_end);
                        let released = self.next_sample_released(sample) * self.release_amplitude;

                        lerp(looped, released, crossfade_pos as f32 / sample.crossfade_release as f32)
                    } else {
                        self.next_sample_released(sample) * self.release_amplitude
                    }
                } else {
                    self.next_sample_released(sample)
                }
            }
            State::ReleasingAfterAttack => {
                let release_length_f = (sample.min_release_length as f64).max(self.release_length);

                if self.audio_position < release_length_f * 2.0 {
                    self.next_sample_normal(sample)
                        * lerp(
                            1.0,
                            0.0,
                            ((self.audio_position - release_length_f) / release_length_f) as f32,
                        )
                } else {
                    0.0
                }
            }
        }
    }
}

#[inline]
fn audio_lookup_with_crossfade(
    position: usize,
    loop_start: usize,
    loop_end: usize,
    buffer: &MonoSample,
    crossfade_buffer: &MonoSample,
) -> f32 {
    if position < loop_end {
        buffer.audio_raw[position]
    } else if position < loop_end + crossfade_buffer.audio_raw.len() {
        crossfade_buffer.audio_raw[position - loop_end]
    } else {
        buffer.audio_raw[((position - loop_start) % (loop_end - loop_start)) + loop_start]
    }
}

fn audio_lookup_with_loop(position: usize, loop_start: usize, loop_end: usize, buffer: &MonoSample) -> f32 {
    if position < loop_end {
        buffer.audio_raw[position]
    } else {
        buffer.audio_raw[((position - loop_start) % (loop_end - loop_start)) + loop_start]
    }
}
