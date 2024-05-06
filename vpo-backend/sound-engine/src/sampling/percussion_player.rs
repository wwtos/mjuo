use crate::{util::interpolate::hermite_lookup, MonoSample, SoundConfig};

use super::{rank::Percussion, Voice};

#[derive(Default, Debug, Clone)]
enum State {
    #[default]
    Uninitialized,
    FadingOut,
    Playing,
    Releasing,
    Stopped,
}

#[derive(Default, Debug, Clone)]
enum QueuedAction {
    Play,
    Release,
    #[default]
    None,
}

#[derive(Default)]
pub struct PercussionPlayer {
    state: State,
    next_state: State,
    queued_action: QueuedAction,

    audio_position: f32,
    resample_ratio: f32,
    fs: f32,

    fade_out_position: f32,
    fade_out_start: f32,
    fade_out_length: f32,

    gain: f32,
    release_gain: f32,
}

impl Voice for PercussionPlayer {
    type Sample = MonoSample;
    type Resource = Percussion;
    type Param = ();

    fn new(resource: &Self::Resource, sample: &Self::Sample, sound_config: SoundConfig) -> Self {
        let fs = sound_config.sample_rate as f32;

        PercussionPlayer {
            state: State::Stopped,
            next_state: State::Stopped,
            queued_action: QueuedAction::None,

            audio_position: 0.0,
            resample_ratio: sample.sample_rate as f32 / fs,
            fs,

            fade_out_position: 0.0,
            fade_out_start: 0.0,
            fade_out_length: resource.release_duration * sound_config.sample_rate as f32,

            gain: resource.gain,
            release_gain: 1.0,
        }
    }

    fn attack(&mut self, _resource: &Self::Resource, _sample: &Self::Sample) {
        match self.state {
            State::Playing => {
                // nothing, as we're already playing
            }
            State::FadingOut => {
                self.queued_action = QueuedAction::Play;
            }
            State::Releasing => {
                // reattack
                self.fade_out_to(State::Playing, 0.0);
            }
            State::Stopped => {
                // attack
                self.state = State::Playing;
                self.audio_position = 0.0;

                self.release_gain = 1.0;
            }
            State::Uninitialized => {}
        }
    }

    fn release(&mut self, _resource: &Self::Resource, _sample: &Self::Sample) {
        match self.state {
            State::Playing => {
                self.state = State::Releasing;
            }
            State::FadingOut => {
                self.queued_action = QueuedAction::Release;
            }
            State::Releasing | State::Stopped => {
                // do nothing
            }
            State::Uninitialized => {}
        }
    }

    fn active(&self) -> bool {
        matches!(self.state, State::Playing | State::FadingOut | State::Releasing)
    }

    fn set_param(&mut self, _param: &Self::Param) {}

    fn reset(&mut self) {
        self.state = State::Stopped;
    }

    fn step(&mut self, resource: &Self::Resource, sample: &Self::Sample) -> f32 {
        match self.state {
            State::Playing => {
                if self.audio_position >= sample.audio_raw.len() as f32 {
                    self.state = State::Stopped;
                }

                self.next_sample_normal(sample) * self.gain
            }
            State::FadingOut => {
                let (out, done) = self.next_sample_fade_out(sample);

                if done {
                    self.state = self.next_state.clone();

                    match self.queued_action {
                        QueuedAction::Play => self.attack(resource, sample),
                        QueuedAction::Release => self.release(resource, sample),
                        QueuedAction::None => {}
                    }

                    self.queued_action = QueuedAction::None;
                }

                out * self.gain
            }
            State::Releasing => {
                self.release_gain -= 1.0 / (self.fs * resource.release_duration);
                self.release_gain = self.release_gain.max(0.0);

                let out = self.next_sample_normal(sample) * self.release_gain;

                if self.release_gain <= 0.0 || self.audio_position >= sample.audio_raw.len() as f32 {
                    self.state = State::Stopped;
                }

                out * self.gain
            }
            State::Stopped => 0.0,
            State::Uninitialized => 0.0,
        }
    }
}

impl PercussionPlayer {
    fn next_sample_normal(&mut self, sample: &MonoSample) -> f32 {
        let out = hermite_lookup(&sample.audio_raw, self.audio_position);

        self.audio_position += self.resample_ratio;

        if self.audio_position >= sample.audio_raw.len() as f32 {
            self.state = State::Stopped;
        }

        out
    }

    fn next_sample_fade_out(&mut self, sample: &MonoSample) -> (f32, bool) {
        let crossfade_factor = ((self.fade_out_position - self.fade_out_start) / self.fade_out_length).min(1.0);

        let old = hermite_lookup(&sample.audio_raw, self.fade_out_position);
        let new = hermite_lookup(&sample.audio_raw, self.audio_position);

        let interpolated = old * (1.0 - crossfade_factor) + new;

        self.audio_position += self.resample_ratio;
        self.fade_out_position += self.resample_ratio;

        (interpolated, crossfade_factor >= 1.0)
    }

    fn fade_out_to(&mut self, next_state: State, new_location: f32) {
        self.queued_action = QueuedAction::None;

        self.state = State::FadingOut;
        self.next_state = next_state;

        self.fade_out_position = self.audio_position;
        self.fade_out_start = self.fade_out_position;
        self.audio_position = new_location;
    }
}
