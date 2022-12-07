use crate::{constants::PI, MonoSample, SoundConfig};

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
    ReAttacking,
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
    release_phase: f32,
    freq_cos_table: Vec<f32>,
    freq_sin_table: Vec<f32>,
}

impl Default for SamplePlayer {
    fn default() -> Self {
        SamplePlayer {
            state: State::Attacking,
            audio_position: 0.0,
            audio_position_release: 0.0,
            stop_at_index: 0,
            release_search_width: 0,
            release_amplitude: 0.0,
            release_length: 0.0,
            global_sample_rate: 0,
            buffer_rate: 0,
            playback_rate: 0.0,
            adjusted_playback_rate: 0.0,
            sample_length: 0,
            release_phase: 0.0,
            freq_cos_table: vec![],
            freq_sin_table: vec![],
        }
    }
}

impl SamplePlayer {
    pub fn new(config: &SoundConfig, sample: &Sample) -> SamplePlayer {
        let buffer_rate = sample.buffer.sample_rate;
        let sample_length = sample.buffer.audio_raw.len();

        // look for potential release locations based on frequency
        let freq = (440.0 / 32.0) * 2_f32.powf((sample.note - 9) as f32 / 12.0);
        let release_search_width = (buffer_rate as f32 / freq) as usize * 2;

        let (freq_cos_table, freq_sin_table) = generate_freq_tables(freq, buffer_rate);

        let release_phase = calc_phase(
            &sample.buffer.audio_raw[sample.release_index..(sample.release_index + freq_cos_table.len())],
            &freq_cos_table,
            &freq_sin_table,
        );

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
            release_phase,
            freq_cos_table,
            freq_sin_table,
        }
    }

    pub fn init(&mut self, config: &SoundConfig, sample: &Sample) {
        let buffer_rate = sample.buffer.sample_rate;
        let sample_length = sample.buffer.audio_raw.len();

        // look for potential release locations based on frequency
        let freq = (440.0 / 32.0) * 2_f32.powf((sample.note - 9) as f32 / 12.0);
        let release_search_width = 2048.max((buffer_rate as f32 / freq) as usize * 2);

        let (freq_cos_table, freq_sin_table) = generate_freq_tables(freq, buffer_rate);

        let release_phase = calc_phase(
            &sample.buffer.audio_raw[sample.release_index..(sample.release_index + freq_cos_table.len())],
            &freq_cos_table,
            &freq_sin_table,
        );

        self.state = State::Attacking;
        self.audio_position = 0.0;
        self.audio_position_release = 0.0;
        self.stop_at_index = 0;
        self.release_search_width = release_search_width;
        self.release_amplitude = 1.0;
        self.release_length = 0.0;
        self.global_sample_rate = config.sample_rate;
        self.buffer_rate = buffer_rate;
        self.playback_rate = 1.0;
        self.adjusted_playback_rate = buffer_rate as f64 / config.sample_rate as f64;
        self.sample_length = sample_length;
        self.release_phase = release_phase;
        self.freq_cos_table = freq_cos_table;
        self.freq_sin_table = freq_sin_table;
    }

    pub fn get_playback_rate(&self) -> f64 {
        self.playback_rate
    }

    pub fn set_playback_rate(&mut self, playback_rate: f64) {
        self.playback_rate = playback_rate;
        self.adjusted_playback_rate = (self.buffer_rate as f64 / self.global_sample_rate as f64) * playback_rate;
    }

    pub fn reset(&mut self) {
        println!("reset");
        self.state = State::Attacking;
        self.audio_position = 0.0;
        self.audio_position_release = 0.0;
        self.stop_at_index = 0;
        self.release_amplitude = 1.0;
    }

    pub fn play(&mut self, sample: &Sample) {
        if self.is_done() {
            self.reset();
        } else {
            let audio = &sample.buffer.audio_raw;

            // what's our current amplitude?
            let current_location = self.get_audio_position() as usize;
            let location_bounded = current_location.max(self.release_search_width);
            let current_amp = rms32(&audio[(location_bounded - self.release_search_width)..location_bounded]);

            if current_amp < 0.01 || current_location + self.freq_cos_table.len() >= audio.len() {
                self.reset();
                return;
            }

            // TODO: binary search?
            // Find place in signal of equal strength
            let mut closest_index = 0;
            let mut closest_score = f32::INFINITY;

            for i in (0..sample.sustain_index).step_by(5) {
                let amp = rms32(&audio[i..(i + self.release_search_width)]);

                let amp_diff = (amp - current_amp).abs();

                if amp_diff < closest_score {
                    closest_index = i;
                    closest_score = amp_diff;
                }
            }

            // next, get it in phase
            let phase_current = calc_phase(
                &audio[current_location..(current_location + self.freq_cos_table.len())],
                &self.freq_cos_table,
                &self.freq_sin_table,
            );

            let phase_new_attack = calc_phase(
                &audio[closest_index..(closest_index + self.freq_cos_table.len())],
                &self.freq_cos_table,
                &self.freq_sin_table,
            );

            let phase_diff = (phase_new_attack - phase_current).rem_euclid(PI * 2.0);
            let attack_shift = ((phase_diff / (PI * 2.0)) * self.freq_cos_table.len() as f32) as usize;

            self.reset();
            self.audio_position_release = current_location as f64;
            self.audio_position = (closest_index + attack_shift) as f64;
            self.stop_at_index = current_location;
            self.state = State::ReAttacking;

            println!("closest in amp: {}", closest_index);
        }
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

        if buffer_position_unbounded >= self.sample_length as i64 || buffer_position_unbounded < 1 {
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

    pub fn get_audio_position(&self) -> f64 {
        match self.state {
            State::Attacking => self.audio_position,
            State::Looping => self.audio_position,
            State::AboutToRelease => self.audio_position,
            State::Releasing => self.audio_position_release,
            State::ReAttacking => self.audio_position,
        }
    }

    pub fn release(&mut self, sample: &Sample) {
        if self.audio_position < 1.0 {
            return;
        }

        let released_at = self.audio_position as usize;
        let release_index = sample.release_index;
        let audio = &sample.buffer.audio_raw;

        if released_at < self.release_search_width {
            return;
        }

        let rms_before = rms32(&audio[(released_at - self.release_search_width)..released_at]);
        let rms_release = rms32(&audio[release_index..(release_index + self.release_search_width)]);

        self.release_amplitude = rms_before / rms_release;

        let phase_stop_at = calc_phase(
            &audio[released_at..(released_at + self.freq_cos_table.len())],
            &self.freq_cos_table,
            &self.freq_sin_table,
        );

        let phase_diff = (self.release_phase - phase_stop_at).rem_euclid(PI * 2.0);
        let release_shift = ((phase_diff / (PI * 2.0)) * self.freq_cos_table.len() as f32) as usize;

        self.stop_at_index = released_at + release_shift;

        self.state = State::AboutToRelease;
    }

    pub fn next_sample(&mut self, sample: &Sample) -> f32 {
        match self.state {
            State::Attacking => {
                if self.audio_position > sample.sustain_index as f64 {
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
            State::ReAttacking => {
                if sample.crossfade > 3 {
                    let crossfade_pos = self.audio_position_release - self.stop_at_index as f64;

                    if (crossfade_pos as usize) < sample.crossfade {
                        let released = self.next_sample_released(sample);
                        let looped = self.next_sample_looped(sample, sample.loop_start, sample.loop_end);

                        lerp(released, looped, crossfade_pos as f32 / sample.crossfade as f32)
                    } else {
                        self.state = State::Attacking;
                        self.next_sample_released(sample)
                    }
                } else {
                    self.next_sample_looped(sample, sample.loop_start, sample.loop_end)
                }
            }
        }
    }

    pub fn is_done(&self) -> bool {
        self.audio_position_release as usize >= self.sample_length - 3
            || self.audio_position as usize >= self.sample_length - 3
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

fn generate_freq_tables(freq: f32, sample_rate: u32) -> (Vec<f32>, Vec<f32>) {
    let cycle_width = sample_rate as f32 / freq;

    let mut cos_table = Vec::new();
    let mut sin_table = Vec::new();

    for i in 0..(cycle_width as usize) {
        cos_table.push(f32::cos((i as f32 / cycle_width) * PI * 2.0));
        sin_table.push(f32::sin((i as f32 / cycle_width) * PI * 2.0));
    }

    (cos_table, sin_table)
}

fn calc_phase(sample: &[f32], cos_table: &[f32], sin_table: &[f32]) -> f32 {
    let mut cos_sum = 0.0;
    let mut sin_sum = 0.0;

    for i in 0..sample.len() {
        cos_sum += sample[i] * cos_table[i];
        sin_sum += sample[i] * sin_table[i];
    }

    f32::atan2(cos_sum, sin_sum)
}
