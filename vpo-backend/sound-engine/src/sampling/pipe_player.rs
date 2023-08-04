use crate::{
    node::filter::{
        FilterSpec,
        FilterType::{self},
        NthBiquadFilter,
    },
    sampling::util::rms32,
    MonoSample,
};

use super::{interpolate::hermite_interpolate, rank::Pipe};

const PHASE_DEBUGGING: bool = false;

#[derive(Debug, Clone)]
pub enum EnvelopeType {
    Attack,
    Release,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    Uninitialized,
    Crossfading,
    Looping,
    Releasing,
    Stopped,
}

#[derive(Debug, Clone)]
enum QueuedAction {
    Play,
    Release,
    None,
}

const ENVELOPE_POINTS: usize = 16;

#[derive(Debug)]
pub struct EnvelopeIndexes {
    pub indexes: [usize; ENVELOPE_POINTS],
    pub peak_amp: f32,
}

#[derive(Debug, Clone)]
pub struct PipePlayer {
    // calculated at the beginning
    freq: f32,
    sample_length: usize,

    state: State,
    next_state: State,
    // in case an action is performed during a crossfade
    queued_action: QueuedAction,

    // basic player values
    audio_position: f32,
    audio_amplitude: f32,

    // voicing
    voicing_amp: f32,
    voicing_comb_coeff: f32, // for even harmonics
    voicing_comb_delay: f32,

    // dynamic air values
    detune: f32,
    gain: f32,
    third_harm_filter: NthBiquadFilter,
    third_db_gain: f32,
    third_spec: FilterSpec<f32>,

    crossfade_position: f32,
    crossfade_start: f32,
    crossfade_length: f32,
}

impl PipePlayer {
    pub fn uninitialized() -> PipePlayer {
        PipePlayer {
            freq: 1.0,
            sample_length: 0,

            state: State::Uninitialized,
            next_state: State::Uninitialized,
            queued_action: QueuedAction::None,

            audio_position: 0.0,
            audio_amplitude: 0.0,

            voicing_amp: 1.0,
            voicing_comb_coeff: 0.0,
            voicing_comb_delay: 1.0,

            detune: 1.0,
            gain: 1.0,
            third_harm_filter: NthBiquadFilter::empty(),
            third_db_gain: 0.0,
            third_spec: FilterSpec::default(),

            crossfade_position: 0.0,
            crossfade_start: 0.0,
            crossfade_length: 0.0,
        }
    }

    pub fn new(pipe: &Pipe, sample: &MonoSample, sample_rate: u32) -> PipePlayer {
        let buffer_rate = sample.sample_rate;
        let sample_length = sample.audio_raw.len();

        // Find different amplitudes in attack section. This allows quickly jumping to a needed
        // amplitude in the attack section (used for reattacking, amplitude is matched with the current
        // audio amplitude in the release phase)
        // TODO: move this to sample loading

        let mut new_player = PipePlayer {
            freq: pipe.freq,
            sample_length,

            state: State::Looping,
            next_state: State::Stopped,
            queued_action: QueuedAction::None,

            audio_position: 1.0,
            audio_amplitude: 1.0,

            voicing_amp: 1.0,
            voicing_comb_coeff: 0.0,
            voicing_comb_delay: 1.0,

            crossfade_position: 1.0,
            crossfade_start: 1.0,
            crossfade_length: pipe.crossfade as f32,

            detune: 1.0,
            gain: 1.0,
            third_harm_filter: NthBiquadFilter::new(
                FilterSpec {
                    f0: sample_rate as f32 / 2.0,
                    fs: sample_rate as f32,
                    filter_type: FilterType::None,
                },
                4,
            ),
            third_db_gain: 0.0,
            third_spec: FilterSpec::new(
                pipe.freq,
                sample_rate as f32,
                FilterType::HighShelf {
                    slope: 1.0,
                    db_gain: 0.0,
                },
            ),
        };

        new_player.calculate_voicing(pipe, sample);

        new_player
    }

    pub fn play(&mut self, pipe: &Pipe, sample: &MonoSample) {
        let current_location = self.audio_position as usize;

        if current_location < 2 {
            return;
        }

        match self.state {
            // Since we were just releasing, this is a case of reattacking
            State::Releasing => {
                let audio = &sample.audio_raw;

                // what's our current amplitude?
                let location_bounded = current_location.max(pipe.amp_window_size);
                let current_amp = rms32(&audio[(location_bounded - pipe.amp_window_size)..location_bounded]);

                if current_amp < 0.01 || current_location + pipe.phase_calculator.window() >= audio.len() {
                    self.reset();
                    return;
                }

                // Find place in attack section of equal strength
                let new_location = envelope_lookup(&pipe.attack_envelope, current_amp);

                // next, get out target in phase
                let attack_shift = pipe.phase_calculator.calc_phase_shift(
                    &audio[current_location..(current_location + pipe.phase_calculator.window())],
                    &audio[new_location..(new_location + pipe.phase_calculator.window())],
                );

                self.crossfade_to(
                    State::Looping,
                    pipe.crossfade as f32,
                    (new_location as f32) + attack_shift,
                );
            }
            State::Crossfading => {
                self.queued_action = QueuedAction::Play;
            }
            State::Stopped => {
                // start over
                self.reset();
            }
            _ => {
                // playing when already playing doesn't do anything
            }
        }
    }

    pub fn release(&mut self, pipe: &Pipe, sample: &MonoSample) {
        match self.state {
            State::Uninitialized => {}
            State::Crossfading => {
                self.queued_action = QueuedAction::Release;
            }
            State::Looping => {
                let current_location = self.audio_position as usize;
                let audio = &sample.audio_raw;

                // what's our current amplitude?
                let location_bounded = current_location.max(pipe.amp_window_size);
                let current_amp = rms32(&audio[(location_bounded - pipe.amp_window_size)..current_location]);

                // Find place in release section of equal strength
                let new_location = envelope_lookup(&pipe.release_envelope, current_amp);

                // next, get out target in phase
                let release_shift = pipe.phase_calculator.calc_phase_shift(
                    &audio[current_location..(current_location + pipe.phase_calculator.window())],
                    &audio[new_location..(new_location + pipe.phase_calculator.window())],
                );

                self.crossfade_to(
                    State::Releasing,
                    pipe.crossfade as f32,
                    new_location as f32 + release_shift,
                );
            }
            State::Releasing | State::Stopped => {}
        }
    }

    pub fn next_sample(&mut self, pipe: &Pipe, sample: &MonoSample) -> f32 {
        let out = match self.state {
            State::Uninitialized => 0.0,
            State::Crossfading => {
                if self.audio_position < (self.sample_length - 3) as f32 {
                    let (out, done) = self.next_sample_crossfade(sample);

                    if done {
                        self.state = self.next_state.clone();
                    }

                    match self.queued_action {
                        QueuedAction::Play => self.play(pipe, sample),
                        QueuedAction::Release => self.release(pipe, sample),
                        QueuedAction::None => {}
                    }

                    self.queued_action = QueuedAction::None;

                    out
                } else {
                    self.state = State::Stopped;

                    0.0
                }
            }
            State::Looping => {
                let out = self.next_sample_normal(sample);

                // loop and crossfade
                if self.audio_position > pipe.loop_end as f32 {
                    let loop_start = pipe.loop_start;
                    let loop_end = pipe.loop_end;

                    let new_location = self.audio_position - (loop_end - loop_start) as f32;

                    self.crossfade_to(State::Looping, pipe.crossfade as f32, new_location);
                }

                out
            }
            State::Releasing => {
                if self.audio_position < (self.sample_length - 3) as f32 {
                    self.next_sample_normal(sample)
                } else {
                    self.state = State::Stopped;

                    0.0
                }
            }
            State::Stopped => 0.0,
        };

        out
    }

    fn next_sample_normal(&mut self, sample: &MonoSample) -> f32 {
        let audio_pos = self.audio_position.max(1.0) as usize;
        let audio_fract = self.audio_position.max(1.0).fract();

        let audio = &sample.audio_raw;

        let x0 = audio[audio_pos - 1];
        let x1 = audio[audio_pos + 0];
        let x2 = audio[audio_pos + 1];
        let x3 = audio[audio_pos + 2];

        self.audio_position += self.detune;

        self.third_harm_filter
            .filter_sample(hermite_interpolate(x0, x1, x2, x3, audio_fract))
            * self.gain
            * self.audio_amplitude
    }

    fn next_sample_crossfade(&mut self, sample: &MonoSample) -> (f32, bool) {
        let crossfade_factor = (self.crossfade_position - self.crossfade_start) / self.crossfade_length;

        let cf_amp = 1.0 - crossfade_factor;
        let aud_amp = crossfade_factor;

        let audio_pos = self.audio_position as usize;
        let crossfade_pos = self.crossfade_position as usize;
        let audio_fract = self.audio_position.max(1.0).fract();

        let audio = &sample.audio_raw;

        let x0 = audio[crossfade_pos - 1] * cf_amp + audio[audio_pos - 1] * aud_amp;
        let x1 = audio[crossfade_pos + 0] * cf_amp + audio[audio_pos + 0] * aud_amp;
        let x2 = audio[crossfade_pos + 1] * cf_amp + audio[audio_pos + 1] * aud_amp;
        let x3 = audio[crossfade_pos + 2] * cf_amp + audio[audio_pos + 2] * aud_amp;

        self.audio_position += self.detune;
        self.crossfade_position += self.detune;

        let out = self
            .third_harm_filter
            .filter_sample(hermite_interpolate(x0, x1, x2, x3, audio_fract))
            * self.gain
            * self.audio_amplitude;

        (out, crossfade_factor >= 1.0)
    }

    fn crossfade_to(&mut self, next_state: State, crossfade_length: f32, new_location: f32) {
        if PHASE_DEBUGGING {
            self.state = next_state;
            self.audio_position = new_location.max(1.0);
        } else {
            self.state = State::Crossfading;
            self.next_state = next_state;

            self.crossfade_position = self.audio_position;
            self.crossfade_start = self.crossfade_position;
            self.crossfade_length = crossfade_length;
            self.audio_position = new_location.max(1.0);
        }
    }

    pub fn is_done(&self) -> bool {
        matches!(self.state, State::Stopped | State::Uninitialized)
    }

    pub fn get_detune(&self) -> f32 {
        self.detune
    }

    pub fn set_detune(&mut self, detune: f32) {
        self.detune = detune;
    }

    pub fn get_gain(&self) -> f32 {
        self.gain
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    pub fn set_shelf_db_gain(&mut self, db_gain: f32) {
        if (db_gain - self.third_db_gain).abs() > 0.5 {
            self.third_spec
                .set_db_gain(db_gain / self.third_harm_filter.get_order_multiplier() as f32);
            self.third_harm_filter.set_spec(self.third_spec.clone());
        }
    }

    pub fn get_position(&self) -> f32 {
        self.audio_position
    }

    pub fn get_crossfade_position(&self) -> Option<f32> {
        if self.state == State::Crossfading {
            Some(self.crossfade_position)
        } else {
            None
        }
    }

    pub fn is_uninitialized(&self) -> bool {
        matches!(self.state, State::Uninitialized)
    }

    pub fn reset(&mut self) {
        self.state = State::Looping;
        self.queued_action = QueuedAction::None;

        self.audio_position = 1.0;
        self.crossfade_position = 1.0;
    }

    fn calculate_voicing(&mut self, pipe: &Pipe, sample: &MonoSample) {
        self.voicing_amp = pipe.amplitude;
        // self.voicing_comb_coeff = pipe.comb
    }
}

fn envelope_lookup(indexes: &EnvelopeIndexes, target_amp: f32) -> usize {
    let closest_env_index =
        ((target_amp / indexes.peak_amp) * ENVELOPE_POINTS as f32).min((ENVELOPE_POINTS - 1) as f32);
    let closest_index = indexes.indexes[closest_env_index.round() as usize];

    closest_index
}

pub fn envelope_points(
    decay_index: usize,
    release_index: usize,
    sample: &MonoSample,
    window_size: usize,
    envelope_type: EnvelopeType,
) -> EnvelopeIndexes {
    let mut envelope_points = [0; ENVELOPE_POINTS];

    let audio = &sample.audio_raw;

    let release_index_capped = release_index.min(audio.len() - window_size);

    let peak_amp = match envelope_type {
        EnvelopeType::Attack => rms32(&audio[decay_index..(decay_index + window_size)]),
        EnvelopeType::Release => rms32(&audio[release_index_capped..(release_index_capped + window_size)]),
    };

    let (amps, offset) = match envelope_type {
        EnvelopeType::Attack => {
            let amps: Vec<f32> = (0..decay_index)
                .step_by(window_size)
                .map(|i| rms32(&audio[i..(i + window_size)]))
                .collect();

            (amps, 0_usize)
        }
        EnvelopeType::Release => {
            let amps: Vec<f32> = (release_index..(audio.len() - window_size))
                .step_by(window_size)
                .map(|i| rms32(&audio[i..(i + window_size)]))
                .collect();

            (amps, release_index)
        }
    };

    for target_amp_index in 0..ENVELOPE_POINTS {
        let target_amp = target_amp_index as f32 / ENVELOPE_POINTS as f32 * peak_amp;

        let mut closest_index = 0;
        let mut closest_score = f32::INFINITY;

        for (i, amp) in amps.iter().enumerate() {
            let amp_diff = (amp - target_amp).abs();

            if amp_diff < closest_score {
                closest_index = i;
                closest_score = amp_diff;
            }
        }

        envelope_points[target_amp_index] = (closest_index * window_size) + offset;
    }

    EnvelopeIndexes {
        indexes: envelope_points,
        peak_amp,
    }
}
