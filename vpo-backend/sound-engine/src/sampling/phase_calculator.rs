use core::f32::consts::PI;
use std::fmt::Debug;

#[derive(Clone)]
pub struct PhaseCalculator {
    cos_sin_table: Vec<(f32, f32)>,
}

impl PhaseCalculator {
    pub fn new(freq: f32, sample_rate: u32) -> PhaseCalculator {
        let one_period_width = sample_rate as f32 / freq;

        let mut table = Vec::with_capacity(one_period_width as usize);

        for i in 0..(one_period_width as usize) {
            table.push((
                f32::cos((i as f32 / one_period_width) * PI * 2.0),
                f32::sin((i as f32 / one_period_width) * PI * 2.0),
            ));
        }

        PhaseCalculator { cos_sin_table: table }
    }

    pub fn empty() -> PhaseCalculator {
        PhaseCalculator {
            cos_sin_table: vec![(0.0, 0.0)],
        }
    }

    pub fn calc_phase(&self, sample: &[f32]) -> f32 {
        // single bin DFT
        let mut cos_sum = 0.0;
        let mut sin_sum = 0.0;

        for i in 0..sample.len() {
            cos_sum += sample[i] * self.cos_sin_table[i].0;
            sin_sum += sample[i] * self.cos_sin_table[i].1;
        }

        f32::atan2(cos_sum, sin_sum)
    }

    /// calculates the needed index offset for `sample_to` in order for continous in phase playback
    pub fn calc_phase_shift(&self, from: usize, to: usize, sample: &[f32]) -> f32 {
        let window = self.window();

        // not enough space to tell
        if from.max(to) + window >= sample.len() {
            return 0.0;
        }

        let phase_from = self.calc_phase(&sample[from..(from + window)]);
        let phase_to = self.calc_phase(&sample[to..(to + window)]);

        let phase_diff = (phase_from - phase_to).rem_euclid(PI * 2.0);

        (phase_diff / (PI * 2.0)) * (window as f32)
    }

    pub fn window(&self) -> usize {
        self.cos_sin_table.len()
    }
}

impl Debug for PhaseCalculator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PhaseCalculator {{ cos_sin_table: [...] }}")
    }
}
