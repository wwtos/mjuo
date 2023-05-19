use crate::{
    node::shelf_filter::{ShelfFilter, ShelfFilterType},
    sampling::util::rms32,
    MonoSample,
};

use super::{interpolate::hermite_interpolate, phase_calculator::PhaseCalculator, rank::Pipe};

const PHASE_DEBUGGING: bool = false;

#[derive(Debug, Clone)]
pub enum EnvelopeType {
    Attack,
    Release,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct PipePlayer {
    // calculated at the beginning
    sample_length: usize,
    attack_peak_amp: f32,
    release_peak_amp: f32,
    amplitude_calc_window: usize,
    phase_calculator: PhaseCalculator,

    state: State,
    next_state: State,
    // in case an action is performed during a crossfade
    queued_action: QueuedAction,

    // basic player values
    audio_position: f32,
    audio_amplitude: f32,

    // dynamic air values
    air_detune: f32,
    air_amplitude: f32,
    shelf_filter: ShelfFilter,

    crossfade_position: f32,
    crossfade_start: f32,
    crossfade_length: f32,

    attack_envelope_indexes: [usize; ENVELOPE_POINTS],
    release_envelope_indexes: [usize; ENVELOPE_POINTS],
}

impl PipePlayer {
    pub fn uninitialized() -> PipePlayer {
        PipePlayer {
            sample_length: 0,
            attack_peak_amp: 0.0,
            release_peak_amp: 0.0,
            amplitude_calc_window: 0,
            phase_calculator: PhaseCalculator::empty(),

            state: State::Uninitialized,
            next_state: State::Uninitialized,
            queued_action: QueuedAction::None,

            audio_position: 0.0,
            audio_amplitude: 0.0,

            air_detune: 1.0,
            air_amplitude: 1.0,
            shelf_filter: ShelfFilter::empty(),

            crossfade_position: 0.0,
            crossfade_start: 0.0,
            crossfade_length: 0.0,

            attack_envelope_indexes: [0; ENVELOPE_POINTS],
            release_envelope_indexes: [0; ENVELOPE_POINTS],
        }
    }

    pub fn new(pipe: &Pipe, sample: &MonoSample, sample_rate: u32) -> PipePlayer {
        let buffer_rate = sample.sample_rate;
        let sample_length = sample.audio_raw.len();

        let freq = (440.0 / 32.0) * 2_f32.powf((pipe.note - 9) as f32 / 12.0 + (pipe.cents as f32 / 1200.0));
        let amplitude_calc_window = (buffer_rate as f32 / freq) as usize * 2;

        let phase_calculator = PhaseCalculator::new(freq, buffer_rate);

        // Find different amplitudes in attack section. This allows quickly jumping to a needed
        // amplitude in the attack section (used for reattacking, amplitude is matched with the current
        // audio amplitude in the release phase)
        let (attack_envelope_indexes, attack_peak_amp) =
            calculate_envelope_points(pipe, sample, amplitude_calc_window, EnvelopeType::Attack);
        let (release_envelope_indexes, release_peak_amp) =
            calculate_envelope_points(pipe, sample, amplitude_calc_window, EnvelopeType::Release);

        PipePlayer {
            sample_length,
            attack_peak_amp,
            release_peak_amp,
            amplitude_calc_window,
            phase_calculator,

            state: State::Looping,
            next_state: State::Stopped,
            queued_action: QueuedAction::None,

            audio_position: 1.0,
            audio_amplitude: 1.0,

            crossfade_position: 1.0,
            crossfade_start: 1.0,
            crossfade_length: 256.0,

            air_detune: 1.0,
            air_amplitude: 1.0,
            shelf_filter: ShelfFilter::new(ShelfFilterType::HighShelf, freq * 2.0, 0.7, 1.0, sample_rate as f32),

            attack_envelope_indexes,
            release_envelope_indexes,
        }
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
                let location_bounded = current_location.max(self.amplitude_calc_window);
                let current_amp = rms32(&audio[(location_bounded - self.amplitude_calc_window)..location_bounded]);

                if current_amp < 0.01 || current_location + self.phase_calculator.window() >= audio.len() {
                    self.reset();
                    return;
                }

                // Find place in attack section of equal strength
                let new_location =
                    envelope_table_lookup(&self.attack_envelope_indexes, current_amp, self.attack_peak_amp);

                // next, get out target in phase
                let attack_shift = self.phase_calculator.calc_phase_shift(
                    &audio[current_location..(current_location + self.phase_calculator.window())],
                    &audio[new_location..(new_location + self.phase_calculator.window())],
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
                let location_bounded = current_location.max(self.amplitude_calc_window);
                let current_amp = rms32(&audio[(location_bounded - self.amplitude_calc_window)..current_location]);

                // Find place in release section of equal strength
                let new_location =
                    envelope_table_lookup(&self.release_envelope_indexes, current_amp, self.release_peak_amp);

                // next, get out target in phase
                let release_shift = self.phase_calculator.calc_phase_shift(
                    &audio[current_location..(current_location + self.phase_calculator.window())],
                    &audio[new_location..(new_location + self.phase_calculator.window())],
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
        match self.state {
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
        }
    }

    fn next_sample_normal(&mut self, sample: &MonoSample) -> f32 {
        let audio_pos = self.audio_position.max(1.0) as usize;
        let audio_fract = self.audio_position.max(1.0).fract();

        let audio = &sample.audio_raw;

        let x0 = audio[audio_pos - 1];
        let x1 = audio[audio_pos + 0];
        let x2 = audio[audio_pos + 1];
        let x3 = audio[audio_pos + 2];

        self.audio_position += self.air_detune;

        self.shelf_filter
            .filter_sample(hermite_interpolate(x0, x1, x2, x3, audio_fract))
            * self.air_amplitude
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

        self.audio_position += self.air_detune;
        self.crossfade_position += self.air_detune;

        let out = self
            .shelf_filter
            .filter_sample(hermite_interpolate(x0, x1, x2, x3, audio_fract))
            * self.air_amplitude
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

    pub fn get_air_detune(&self) -> f32 {
        self.air_detune
    }

    pub fn set_air_detune(&mut self, detune: f32) {
        self.air_detune = detune;
    }

    pub fn get_air_amplitude(&self) -> f32 {
        self.air_amplitude
    }

    pub fn set_air_amplitude(&mut self, amplitude: f32) {
        self.air_amplitude = amplitude;
    }

    pub fn set_shelf_gain(&mut self, gain: f32) {
        if (gain - self.shelf_filter.get_gain()).abs() > 0.05 {
            self.shelf_filter.set_gain(gain);
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
}

fn envelope_table_lookup(table: &[usize; ENVELOPE_POINTS], target_amp: f32, peak_amp: f32) -> usize {
    let closest_env_index = ((target_amp / peak_amp) * ENVELOPE_POINTS as f32).min((ENVELOPE_POINTS - 1) as f32);
    let closest_index = table[closest_env_index.round() as usize];

    closest_index
}

fn calculate_envelope_points(
    pipe: &Pipe,
    sample: &MonoSample,
    window_size: usize,
    envelope_type: EnvelopeType,
) -> ([usize; ENVELOPE_POINTS], f32) {
    let mut envelope_points = [0; ENVELOPE_POINTS];

    let audio = &sample.audio_raw;

    let peak_amp = match envelope_type {
        EnvelopeType::Attack => rms32(&audio[pipe.decay_index..(pipe.decay_index + window_size)]),
        EnvelopeType::Release => rms32(&audio[pipe.release_index..(pipe.release_index + window_size)]),
    };

    let (amps, offset) = match envelope_type {
        EnvelopeType::Attack => {
            let amps: Vec<f32> = (0..pipe.decay_index)
                .step_by(window_size)
                .map(|i| rms32(&audio[i..(i + window_size)]))
                .collect();

            (amps, 0_usize)
        }
        EnvelopeType::Release => {
            let amps: Vec<f32> = (pipe.release_index..(audio.len() - window_size))
                .step_by(window_size)
                .map(|i| rms32(&audio[i..(i + window_size)]))
                .collect();

            (amps, pipe.release_index)
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

    (envelope_points, peak_amp)
}
