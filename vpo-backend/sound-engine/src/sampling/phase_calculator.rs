use core::f32::consts::PI;

#[derive(Debug, Clone)]
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
        // calculate phase using methods from the continous wavelet transform
        let mut cos_sum = 0.0;
        let mut sin_sum = 0.0;

        for i in 0..sample.len() {
            cos_sum += sample[i] * self.cos_sin_table[i].0;
            sin_sum += sample[i] * self.cos_sin_table[i].1;
        }

        f32::atan2(cos_sum, sin_sum)
    }

    pub fn window(&self) -> usize {
        self.cos_sin_table.len()
    }
}