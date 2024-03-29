use crate::{
    node::filter::{
        FilterSpec,
        FilterType::{self},
        NthBiquadFilter, SimpleComb,
    },
    sampling::util::rms32,
    util::interpolate::hermite_lookup,
    MonoSample,
};

use super::rank::Pipe;

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
    sample_length: usize,

    state: State,
    next_state: State,
    // in case an action is performed during a crossfade
    queued_action: QueuedAction,

    // basic player values
    audio_position: f32,
    resample_ratio: f32,

    // voicing
    voicing_amp: f32,
    voicing_comb: SimpleComb,

    // dynamic air values
    detune: f32,
    gain: f32,
    third_harm_filter: NthBiquadFilter<4>,
    third_db_gain: f32,
    third_spec: FilterSpec<f32>,

    crossfade_position: f32,
    crossfade_start: f32,
    crossfade_length: f32,
}

impl PipePlayer {
    pub fn uninitialized() -> PipePlayer {
        PipePlayer {
            sample_length: 0,

            state: State::Uninitialized,
            next_state: State::Uninitialized,
            queued_action: QueuedAction::None,

            audio_position: 0.0,
            resample_ratio: 0.0,

            voicing_amp: 1.0,
            voicing_comb: SimpleComb::default(),

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
        let sample_length = sample.audio_raw.len();

        let mut new_player = PipePlayer {
            sample_length,

            state: State::Looping,
            next_state: State::Stopped,
            queued_action: QueuedAction::None,

            audio_position: 0.0,
            resample_ratio: sample.sample_rate as f32 / sample_rate as f32,

            voicing_amp: pipe.amplitude,
            voicing_comb: SimpleComb::default(),

            crossfade_position: 0.0,
            crossfade_start: 0.0,
            crossfade_length: pipe.crossfade as f32,

            detune: 1.0,
            gain: 1.0,
            third_harm_filter: NthBiquadFilter::new(FilterSpec {
                f0: sample_rate as f32 / 2.0,
                fs: sample_rate as f32,
                filter_type: FilterType::None,
            }),
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

        match self.state {
            State::Uninitialized => {}
            // Since we were just releasing, this is a case of reattacking
            State::Releasing => {
                let audio = &sample.audio_raw;

                // what's our current amplitude?
                let location_bounded = current_location.max(pipe.amp_window_size);
                let current_amp = rms32(&audio[(location_bounded - pipe.amp_window_size)..location_bounded]);

                // quiet enough that we should just restart
                if current_amp < 0.01 || current_location + pipe.phase_calculator.window() >= audio.len() {
                    self.restart();
                    return;
                }

                // Find place in attack section of equal strength
                let new_location = envelope_lookup(&pipe.attack_envelope, current_amp);

                // next, get out target in phase
                let attack_shift = pipe
                    .phase_calculator
                    .calc_phase_shift(current_location, new_location, audio);

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
                self.restart();
            }
            State::Looping => {
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
                let current_amp = rms32(&audio[(location_bounded - pipe.amp_window_size)..location_bounded]);

                // Find place in release section of equal strength
                let new_location = envelope_lookup(&pipe.release_envelope, current_amp);

                // next, get out target in phase
                let release_shift = pipe
                    .phase_calculator
                    .calc_phase_shift(current_location, new_location, audio);

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

                        match self.queued_action {
                            QueuedAction::Play => self.play(pipe, sample),
                            QueuedAction::Release => self.release(pipe, sample),
                            QueuedAction::None => {}
                        }

                        self.queued_action = QueuedAction::None;
                    }

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
        let comb_pass = self.voicing_comb.filter(
            audio_lookup(&sample.audio_raw, self.audio_position),
            &sample.audio_raw,
            self.audio_position,
        );

        self.audio_position += self.detune * self.resample_ratio;

        self.third_harm_filter.filter_sample(comb_pass) * self.gain * self.voicing_amp
    }

    fn next_sample_crossfade(&mut self, sample: &MonoSample) -> (f32, bool) {
        let crossfade_factor = (self.crossfade_position - self.crossfade_start) / self.crossfade_length;

        let old = self.voicing_comb.filter(
            audio_lookup(&sample.audio_raw, self.crossfade_position),
            &sample.audio_raw,
            self.crossfade_position,
        );

        let new = self.voicing_comb.filter(
            audio_lookup(&sample.audio_raw, self.audio_position),
            &sample.audio_raw,
            self.audio_position,
        );

        let interpolated = old * (1.0 - crossfade_factor) + new * crossfade_factor;

        self.audio_position += self.detune * self.resample_ratio;
        self.crossfade_position += self.detune * self.resample_ratio;

        let out = self.third_harm_filter.filter_sample(interpolated) * self.gain * self.voicing_amp;

        (out, crossfade_factor >= 1.0)
    }

    fn crossfade_to(&mut self, next_state: State, crossfade_length: f32, new_location: f32) {
        if PHASE_DEBUGGING {
            self.state = next_state;
            self.audio_position = new_location;
        } else {
            self.queued_action = QueuedAction::None;

            self.state = State::Crossfading;
            self.next_state = next_state;

            self.crossfade_position = self.audio_position;
            self.crossfade_start = self.crossfade_position;
            self.crossfade_length = crossfade_length;
            self.audio_position = new_location;
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
        // avoid unnecessary calculations if they're too small to hear
        // TODO: this 0.5 should probably be a constant
        if (db_gain - self.third_db_gain).abs() > 0.5 {
            self.third_spec
                .set_db_gain(db_gain / self.third_harm_filter.get_order_multiplier() as f32);
            self.third_harm_filter.set_spec(self.third_spec.clone());
            self.third_db_gain = db_gain;
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

    pub fn restart(&mut self) {
        self.state = State::Looping;
        self.queued_action = QueuedAction::None;

        self.audio_position = 1.0;
        self.crossfade_position = 1.0;
    }

    fn calculate_voicing(&mut self, pipe: &Pipe, sample: &MonoSample) {
        self.voicing_comb = SimpleComb::new(pipe.freq * 2.0, sample.sample_rate as f32, -pipe.comb_coeff);
        self.voicing_amp = 1.0 / self.voicing_comb.response(pipe.freq, sample.sample_rate as f32) * pipe.amplitude;

        self.third_spec.f0 = pipe.freq;
        // recalculate the filter coefficients
        self.set_shelf_db_gain(self.third_db_gain);
    }
}

#[inline]
fn audio_lookup(sample: &[f32], position: f32) -> f32 {
    let sample_index = position as usize;

    if sample_index > sample.len() - 4 {
        0.0
    } else {
        hermite_lookup(position, sample)
    }
}

fn envelope_lookup(indexes: &EnvelopeIndexes, target_amp: f32) -> usize {
    let closest_env_index =
        ((target_amp / indexes.peak_amp) * ENVELOPE_POINTS as f32).min((ENVELOPE_POINTS - 1) as f32);
    let closest_index = indexes.indexes[closest_env_index.round() as usize];

    closest_index
}

pub fn envelope_indexes(
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
