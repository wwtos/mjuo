use crate::{
    constants::PI,
    sampling::util::{amp32, rms32, s_mult},
    MonoSample,
};

use super::{
    interpolate::{hermite_interpolate, lerp},
    phase_calculator::PhaseCalculator,
    rank::Pipe,
};

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

const ENVELOPE_POINTS: usize = 8;

#[derive(Debug, Clone)]
pub struct PipePlayer {
    sample_length: usize,
    max_amp: f32,
    amplitude_calc_window: usize,
    phase_of_release: f32,
    phase_calculator: PhaseCalculator,

    state: State,
    next_state: State,
    // in case an action is performed during a crossfade
    queued_action: QueuedAction,

    audio_position: f32,
    audio_amplitude: f32,
    playback_rate: f32,

    crossfade_position: f32,
    crossfade_amplitude: f32,
    crossfade_start: f32,
    crossfade_length: f32,

    attack_envelope_indexes: [usize; ENVELOPE_POINTS],
}

impl PipePlayer {
    pub fn uninitialized() -> PipePlayer {
        PipePlayer {
            sample_length: 0,
            max_amp: 0.0,
            amplitude_calc_window: 0,
            phase_of_release: 0.0,
            phase_calculator: PhaseCalculator::empty(),
            state: State::Uninitialized,
            next_state: State::Uninitialized,
            queued_action: QueuedAction::None,
            audio_position: 0.0,
            audio_amplitude: 0.0,
            playback_rate: 0.0,
            crossfade_position: 0.0,
            crossfade_amplitude: 0.0,
            crossfade_start: 0.0,
            crossfade_length: 0.0,
            attack_envelope_indexes: [0; ENVELOPE_POINTS],
        }
    }

    pub fn new(pipe: &Pipe, sample: &MonoSample) -> PipePlayer {
        let buffer_rate = sample.sample_rate;
        let sample_length = sample.audio_raw.len();
        let audio = &sample.audio_raw;

        let freq = (440.0 / 32.0) * 2_f32.powf((pipe.note - 9) as f32 / 12.0 + (pipe.cents as f32 / 1200.0));
        let amplitude_calc_window = (buffer_rate as f32 / freq) as usize * 2;

        let phase_calculator = PhaseCalculator::new(freq, buffer_rate);

        let release_phase =
            phase_calculator.calc_phase(&audio[pipe.release_index..(pipe.release_index + phase_calculator.window())]);

        // Find different amplitudes in attack section. This allows quickly jumping to a needed
        // amplitude in the attack section (used for reattacking, amplitude is matched with the current
        // audio amplitude in the release phase)
        let mut envelope_points = [0; ENVELOPE_POINTS];
        let peak_amp = amp32(&audio[pipe.decay_index..(pipe.decay_index + amplitude_calc_window)]);

        for target_amp_index in 0..ENVELOPE_POINTS {
            let target_amp = target_amp_index as f32 / ENVELOPE_POINTS as f32 * peak_amp;

            let mut closest_index = 0;
            let mut closest_score = f32::INFINITY;

            for i in (0..pipe.decay_index).step_by(5) {
                let amp = amp32(&audio[i..(i + amplitude_calc_window)]);

                let amp_diff = (amp - target_amp).abs();

                if amp_diff < closest_score {
                    closest_index = i;
                    closest_score = amp_diff;
                }
            }

            envelope_points[target_amp_index] = closest_index;
        }

        PipePlayer {
            sample_length,
            max_amp: peak_amp,
            amplitude_calc_window,
            phase_of_release: release_phase,
            phase_calculator,
            state: State::Looping,
            next_state: State::Stopped,
            queued_action: QueuedAction::None,
            audio_position: 1.0,
            audio_amplitude: 1.0,
            crossfade_position: 1.0,
            crossfade_amplitude: 1.0,
            crossfade_start: 1.0,
            crossfade_length: 256.0,
            playback_rate: 1.0,
            attack_envelope_indexes: envelope_points,
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
                let current_amp = amp32(&s_mult(
                    &audio[(location_bounded - self.amplitude_calc_window)..location_bounded],
                    self.audio_amplitude,
                ));

                if current_amp < 0.01 || current_location + self.phase_calculator.window() >= audio.len() {
                    self.reset();
                    return;
                }

                // Find place in attack section of equal strength
                let closest_env_index =
                    ((current_amp / self.max_amp) * ENVELOPE_POINTS as f32).min((ENVELOPE_POINTS - 1) as f32);
                let closest_index = lerp(
                    self.attack_envelope_indexes[closest_env_index.floor() as usize] as f32,
                    self.attack_envelope_indexes[closest_env_index.ceil() as usize] as f32,
                    closest_env_index - closest_env_index.floor(),
                ) as usize;

                // next, get it in phase
                let phase_of_current = self
                    .phase_calculator
                    .calc_phase(&audio[current_location..(current_location + self.phase_calculator.window())]);

                let phase_of_target = self
                    .phase_calculator
                    .calc_phase(&audio[closest_index..(closest_index + self.phase_calculator.window())]);

                let phase_diff = (phase_of_current - phase_of_target).rem_euclid(PI * 2.0);
                let attack_shift = (phase_diff / (PI * 2.0)) * self.phase_calculator.window() as f32;

                self.crossfade_to(
                    State::Looping,
                    pipe.crossfade as f32,
                    (closest_index as f32) + attack_shift,
                );

                self.crossfade_amplitude = self.audio_amplitude;
                self.audio_amplitude = 1.0;
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
                let release_index = pipe.release_index;
                let audio = &sample.audio_raw;

                let location_bounded = current_location.max(self.amplitude_calc_window);
                let amp_current = rms32(&audio[(location_bounded - self.amplitude_calc_window)..current_location]);
                let amp_of_release = rms32(&audio[release_index..(release_index + self.amplitude_calc_window)]);

                let release_amp_adjustment = amp_current / amp_of_release;

                let phase_of_current = self
                    .phase_calculator
                    .calc_phase(&audio[current_location..(current_location + self.phase_calculator.window())]);

                let phase_diff = (phase_of_current - self.phase_of_release).rem_euclid(PI * 2.0);
                let release_shift = (phase_diff / (PI * 2.0)) * self.phase_calculator.window() as f32;

                self.crossfade_to(
                    State::Releasing,
                    pipe.crossfade as f32,
                    pipe.release_index as f32 + release_shift,
                );

                self.crossfade_amplitude = 1.0;
                self.audio_amplitude = release_amp_adjustment;
            }
            State::Releasing | State::Stopped => {}
        }
    }

    pub fn next_sample(&mut self, pipe: &Pipe, sample: &MonoSample) -> f32 {
        match self.state {
            State::Uninitialized => 0.0,
            State::Crossfading => {
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
        let audio_pos = self.audio_position as usize;

        let audio = &sample.audio_raw;

        let x0 = audio[audio_pos - 1] * self.audio_amplitude;
        let x1 = audio[audio_pos + 0] * self.audio_amplitude;
        let x2 = audio[audio_pos + 1] * self.audio_amplitude;
        let x3 = audio[audio_pos + 2] * self.audio_amplitude;

        self.audio_position += self.playback_rate;

        hermite_interpolate(x0, x1, x2, x3, self.audio_position.fract())
    }

    fn next_sample_crossfade(&mut self, sample: &MonoSample) -> (f32, bool) {
        let crossfade_factor = (self.crossfade_position - self.crossfade_start) / self.crossfade_length;

        let cf_amp = (1.0 - crossfade_factor) * self.crossfade_amplitude;
        let aud_amp = crossfade_factor * self.audio_amplitude;

        let audio_pos = self.audio_position as usize;
        let crossfade_pos = self.crossfade_position as usize;

        let audio = &sample.audio_raw;

        let x0 = audio[crossfade_pos - 1] * cf_amp + audio[audio_pos - 1] * aud_amp;
        let x1 = audio[crossfade_pos + 0] * cf_amp + audio[audio_pos + 0] * aud_amp;
        let x2 = audio[crossfade_pos + 1] * cf_amp + audio[audio_pos + 1] * aud_amp;
        let x3 = audio[crossfade_pos + 2] * cf_amp + audio[audio_pos + 2] * aud_amp;

        self.audio_position += self.playback_rate;
        self.crossfade_position += self.playback_rate;

        let out = hermite_interpolate(x0, x1, x2, x3, self.audio_position.fract());

        (out, crossfade_factor >= 1.0)
    }

    fn crossfade_to(&mut self, next_state: State, crossfade_length: f32, new_location: f32) {
        self.state = State::Crossfading;
        self.next_state = next_state;

        self.crossfade_position = self.audio_position;
        self.crossfade_start = self.crossfade_position;
        self.crossfade_length = crossfade_length;
        self.audio_position = new_location;
    }

    pub fn is_done(&self) -> bool {
        matches!(self.state, State::Stopped | State::Uninitialized)
    }

    pub fn get_playback_rate(&self) -> f32 {
        self.playback_rate
    }

    pub fn set_playback_rate(&mut self, playback_rate: f32) {
        self.playback_rate = playback_rate;
    }

    pub fn is_uninitialized(&self) -> bool {
        matches!(self.state, State::Uninitialized)
    }

    pub fn reset(&mut self) {
        self.state = State::Looping;
        self.queued_action = QueuedAction::None;

        self.audio_position = 1.0;
        self.crossfade_position = 1.0;
        self.audio_amplitude = 1.0;
        self.crossfade_amplitude = 1.0;
    }
}
