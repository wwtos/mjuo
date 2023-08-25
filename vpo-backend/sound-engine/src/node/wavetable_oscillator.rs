use crate::{util::interpolate::lerp, MonoSample, SoundConfig};

#[derive(Debug, Clone)]
pub struct WavetableOscillator {
    phase: f32,
    frequency: f32,
    sample_rate: u32,
}

impl WavetableOscillator {
    pub fn new(sound_config: SoundConfig) -> WavetableOscillator {
        WavetableOscillator {
            phase: 0_f32,
            frequency: 440_f32,
            sample_rate: sound_config.sample_rate,
        }
    }

    pub fn new_with_frequency(
        sound_config: SoundConfig,
        wavetable: &MonoSample,
        frequency: f32,
    ) -> WavetableOscillator {
        let mut oscillator = WavetableOscillator::new(sound_config);
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
        self.phase = self.phase.fract(); // keep it bounded

        let pos = self.phase * wavetable.audio_raw.len() as f32;

        lerp(
            wavetable.audio_raw[pos as usize],
            wavetable.audio_raw[(pos as usize + 1) % wavetable.audio_raw.len()],
            pos.fract(),
        )
    }

    pub fn get_frequency(&self) -> f32 {
        self.frequency
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
}
