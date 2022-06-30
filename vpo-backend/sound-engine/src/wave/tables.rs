use crate::constants::{PI, SAMPLE_RATE, TWO_PI};
use lazy_static::lazy_static;

pub const WAVETABLE_SIZE: usize = 256;
pub const BASE_FREQUENCY: f32 = 8.0;
pub const FREQUENCY_STEPS: usize = 5000;

lazy_static! {
    pub static ref SINE_VALUES: Vec<[f32; WAVETABLE_SIZE]> = {
        let mut wavetables = vec![[0_f32; WAVETABLE_SIZE]; FREQUENCY_STEPS];

        // allow for brevity, iterators make it unnecessarily confusing
        #[allow(clippy::needless_range_loop)]
        for i in 0..FREQUENCY_STEPS {
            for j in 0..WAVETABLE_SIZE {
                wavetables[i][j] = ((j as f32 / WAVETABLE_SIZE as f32) * TWO_PI).sin()
            }
        }

        wavetables
    };

    pub static ref SAWTOOTH_VALUES: Vec<[f32; WAVETABLE_SIZE]> = {
        let mut wavetables = vec![[0_f32; WAVETABLE_SIZE]; FREQUENCY_STEPS];

        // allow for brevity, iterators make it unnecessarily confusing
        #[allow(clippy::needless_range_loop)]
        for i in 0..FREQUENCY_STEPS {
            let freq = BASE_FREQUENCY * (i + 1) as f32;
            let num_harmonics = ((SAMPLE_RATE / 2) as f32 / freq) as i32; // rounded down

            for j in 0..WAVETABLE_SIZE {
                let phase = j as f32 / WAVETABLE_SIZE as f32 * TWO_PI;

                let mut sin_sum = 0.0;

                for harmonic_index in 1..num_harmonics {
                    sin_sum += f32::sin(phase * harmonic_index as f32) / harmonic_index as f32;
                }

                //adjust the volume
                wavetables[i][j] = sin_sum * 2.0 / PI;
            }
        }

        wavetables
    };

    pub static ref SQUARE_VALUES: Vec<[f32; WAVETABLE_SIZE]> = {
        let mut wavetables = vec![[0_f32; WAVETABLE_SIZE]; FREQUENCY_STEPS];

        // allow for brevity, iterators make it unnecessarily confusing
        #[allow(clippy::needless_range_loop)]
        for i in 0..FREQUENCY_STEPS {
            let freq = BASE_FREQUENCY * (i + 1) as f32;
            let num_harmonics = ((SAMPLE_RATE / 2) as f32 / freq) as i32; // rounded down

            for j in 0..WAVETABLE_SIZE {
                let phase = j as f32 / WAVETABLE_SIZE as f32 * TWO_PI;

                let mut sin_sum = 0.0;

                for harmonic_index in 1..num_harmonics {
                    if harmonic_index % 2 == 1 {
                        sin_sum += f32::sin(phase * harmonic_index as f32) / harmonic_index as f32;
                    }
                }

                //adjust the volume
                wavetables[i][j] = sin_sum * 4.0 / PI;
            }
        }

        wavetables
    };

    pub static ref TRIANGLE_VALUES: Vec<[f32; WAVETABLE_SIZE]> = {
        let mut wavetables = vec![[0_f32; WAVETABLE_SIZE]; FREQUENCY_STEPS];

        // allow for brevity, iterators make it unnecessarily confusing
        #[allow(clippy::needless_range_loop)]
        for i in 0..FREQUENCY_STEPS {
            let freq = BASE_FREQUENCY * (i + 1) as f32;
            let num_harmonics = ((SAMPLE_RATE / 2) as f32 / freq) as i32; // rounded down

            for j in 0..WAVETABLE_SIZE {
                let phase = j as f32 / WAVETABLE_SIZE as f32 * PI;

                let mut sin_sum = 0.0;

                for harmonic_index in 1..num_harmonics {
                    if harmonic_index % 4 == 1 {
                        sin_sum += f32::sin(phase * harmonic_index as f32) / (harmonic_index * harmonic_index) as f32;
                    } else if harmonic_index % 4 == 3 {
                        sin_sum -= f32::sin(phase * harmonic_index as f32) / (harmonic_index * harmonic_index) as f32;
                    }
                }

                //adjust the volume
                wavetables[i][j] = sin_sum * 4.0 / PI;
            }
        }

        wavetables
    };
}
