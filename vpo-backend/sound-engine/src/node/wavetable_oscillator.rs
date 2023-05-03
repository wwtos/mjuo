use crate::{wave::interpolate::interpolate, MonoSample, SoundConfig};

use super::biquad_filter::{BiquadFilter, BiquadFilterType};

#[derive(Debug, Clone)]
pub struct WavetableOscillator {
    phase: f32,
    frequency: f32,
    filter: BiquadFilter,
    sample_rate: u32,
    sample_freq: f32,
}

impl WavetableOscillator {
    pub fn new(sound_config: SoundConfig, wavetable: &MonoSample) -> WavetableOscillator {
        let sample_freq = sound_config.sample_rate as f32 / wavetable.audio_raw.len() as f32;

        WavetableOscillator {
            phase: 0_f32,
            frequency: 440_f32,
            sample_rate: sound_config.sample_rate,
            sample_freq,
            filter: BiquadFilter::new(
                &sound_config,
                BiquadFilterType::Lowpass,
                sound_config.sample_rate as f32 / 2.0,
                0.7,
            ),
        }
    }

    pub fn new_with_frequency(
        sound_config: SoundConfig,
        wavetable: &MonoSample,
        frequency: f32,
    ) -> WavetableOscillator {
        let mut oscillator = WavetableOscillator::new(sound_config, wavetable);
        oscillator.set_frequency(frequency);

        oscillator
    }

    pub fn get_phase(&self) -> f32 {
        self.phase
    }

    pub fn set_phase(&mut self, phase: f32) {
        self.phase = phase;
    }

    #[inline]
    pub fn get_next_sample(&mut self, wavetable: &MonoSample) -> f32 {
        let phase_advance = self.frequency / (self.sample_rate as f32);

        self.phase += phase_advance;
        self.phase -= self.phase.floor(); // keep it bounded

        interpolate(&wavetable.audio_raw, &mut self.filter, self.phase)
    }

    pub fn get_frequency(&self) -> f32 {
        self.frequency
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;

        let wavetable_filter_freq =
            ((self.sample_freq / self.frequency) * (self.sample_rate as f32 / 2.0)).min(self.sample_rate as f32 / 2.1);

        self.filter.set_frequency(wavetable_filter_freq);
    }
}
