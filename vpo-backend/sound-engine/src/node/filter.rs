use std::{
    array::from_fn,
    f32::consts::PI,
    f64::consts::TAU,
    iter::{repeat, repeat_with, Sum},
};

use num::Float;
use smallvec::SmallVec;

use crate::util::interpolate::lerp;

#[derive(Debug, Clone)]
pub struct FilterCoeffs<const N: usize, F: Float> {
    pub b: [F; N],
    pub a: [F; N],
}

impl<const N: usize, F: Float> Default for FilterCoeffs<N, F> {
    fn default() -> Self {
        let mut b = [F::zero(); N];
        let mut a = [F::zero(); N];

        if N > 0 {
            b[0] = F::one();
            a[0] = F::one();
        }

        Self { b, a }
    }
}

#[derive(Debug, Clone)]
pub enum FilterType<F: Float> {
    LowPass { q: F },
    HighPass { q: F },
    BandPass { bandwidth: F },
    Notch { bandwidth: F },
    AllPass { q: F },
    Peaking { bandwidth: F, db_gain: F },
    LowShelf { slope: F, db_gain: F },
    HighShelf { slope: F, db_gain: F },
    None,
}

impl<F: Float> Default for FilterType<F> {
    fn default() -> Self {
        FilterType::None
    }
}

#[derive(Debug, Clone, Default)]
pub struct FilterSpec<F: Float> {
    pub f0: F,
    pub fs: F,
    pub filter_type: FilterType<F>,
}

impl<F: Float> FilterSpec<F> {
    pub fn new(f0: F, fs: F, filter_type: FilterType<F>) -> FilterSpec<F> {
        FilterSpec { f0, fs, filter_type }
    }

    pub fn none() -> FilterSpec<F> {
        FilterSpec {
            f0: F::zero(),
            fs: F::zero(),
            filter_type: FilterType::None,
        }
    }

    pub fn set_db_gain(&mut self, new_db_gain: F) -> bool {
        match &mut self.filter_type {
            FilterType::Peaking { db_gain, .. }
            | FilterType::LowShelf { db_gain, .. }
            | FilterType::HighShelf { db_gain, .. } => {
                *db_gain = new_db_gain;

                true
            }
            _ => false,
        }
    }
}

pub fn bar() {
    println!("something");
}

#[allow(non_snake_case)]
pub fn bandwidth_to_q<F: Float>(bandwidth: F, ω0: F) -> F {
    let n2 = F::one() + F::one();

    let one_over_Q = n2 * F::sinh((n2.ln() / n2) * bandwidth * (ω0 / ω0.sin()));

    F::one() / one_over_Q
}

#[allow(non_snake_case)]
pub fn slope_to_q<F: Float>(db_gain: F, slope: F) -> F {
    let n1 = F::one();
    let n2 = n1 + n1;
    let n10 = F::from(10).unwrap();
    let n40 = F::from(40).unwrap();

    let A = n10.powf(db_gain / n40);

    let one_over_Q = F::sqrt((A + n1 / A) * (n1 / slope - n1) + n2);

    n1 / one_over_Q
}

fn db_gain_to_a<F: Float>(db_gain: F) -> F {
    //      _______________
    // A = √10^(db_gain/20)
    //
    // aka A = 10^(db_gain/40)
    let n10 = F::from(10).unwrap();
    let n40 = F::from(40).unwrap();

    n10.powf(db_gain / n40)
}

// https://www.w3.org/TR/audio-eq-cookbook/
#[allow(non_snake_case)]
pub fn filter_coeffs<F: Float>(spec: FilterSpec<F>) -> FilterCoeffs<3, F> {
    let two_pi = F::from(TAU).unwrap();

    let ω0 = two_pi * spec.f0 / spec.fs;

    let cos_ω0 = ω0.cos();
    let sin_ω0 = ω0.sin();

    let mut b = [F::zero(); 3];
    let mut a = [F::zero(); 3];

    let n0 = F::zero();
    let n1 = F::one();
    let n2 = n1 + n1;

    let α = match spec.filter_type {
        FilterType::LowPass { q } | FilterType::HighPass { q } | FilterType::AllPass { q } => sin_ω0 / (n2 * q),
        FilterType::BandPass { bandwidth }
        | FilterType::Notch { bandwidth }
        | FilterType::Peaking { bandwidth, .. } => sin_ω0 * F::sinh(n2.ln() / n2 * bandwidth * ω0 / sin_ω0),
        FilterType::LowShelf { slope, db_gain } | FilterType::HighShelf { slope, db_gain } => {
            let A = db_gain_to_a(db_gain);

            sin_ω0 / n2 * F::sqrt((A + n1 / A) * (n1 / slope - n1) + n2)
        }
        FilterType::None => n0,
    };

    match spec.filter_type {
        FilterType::LowPass { .. } => {
            b[0] = (n1 - cos_ω0) / n2;
            b[1] = n1 - cos_ω0;
            b[2] = (n1 - cos_ω0) / n2;

            a[0] = n1 + α;
            a[1] = -n2 * cos_ω0;
            a[2] = n1 - α;
        }
        FilterType::HighPass { .. } => {
            b[0] = (n1 + cos_ω0) / n2;
            b[1] = -(n1 + cos_ω0);
            b[2] = (n1 + cos_ω0) / n2;

            a[0] = n1 + α;
            a[1] = -n2 * cos_ω0;
            a[2] = n1 - α;
        }
        FilterType::BandPass { .. } => {
            b[0] = α;
            b[1] = n0;
            b[2] = -α;

            a[0] = n1 + α;
            a[1] = -n2 * cos_ω0;
            a[2] = n1 - α;
        }
        FilterType::Notch { .. } => {
            b[0] = n1;
            b[1] = -n2 * cos_ω0;
            b[2] = n1;

            a[0] = n1 + α;
            a[1] = -n2 * cos_ω0;
            a[2] = n1 - α;
        }
        FilterType::AllPass { .. } => {
            b[0] = n1 - α;
            b[1] = -n2 * cos_ω0;
            b[2] = n1 + α;

            a[0] = n1 + α;
            a[1] = -n2 * cos_ω0;
            a[2] = n1 - α;
        }
        FilterType::Peaking { db_gain, .. } => {
            let A = db_gain_to_a(db_gain);

            b[0] = n1 + α * A;
            b[1] = -n2 * cos_ω0;
            b[2] = n1 - α * A;

            a[0] = n1 + (α / A);
            a[1] = -n2 * cos_ω0;
            a[2] = n1 - (α / A);
        }
        FilterType::LowShelf { db_gain, .. } => {
            let A = db_gain_to_a(db_gain);
            let A_sqrt = A.sqrt();

            b[0] = A * ((A + n1) - (A - n1) * cos_ω0 + n2 * A_sqrt * α);
            b[1] = n2 * A * ((A - n1) - (A + n1) * cos_ω0);
            b[2] = A * ((A + n1) - (A - n1) * cos_ω0 - n2 * A_sqrt * α);

            a[0] = (A + n1) + (A - n1) * cos_ω0 + n2 * A_sqrt * α;
            a[1] = -n2 * ((A - n1) + (A + n1) * cos_ω0);
            a[2] = (A + n1) + (A - n1) * cos_ω0 - n2 * A_sqrt * α;
        }
        FilterType::HighShelf { db_gain, .. } => {
            let A = db_gain_to_a(db_gain);
            let A_sqrt = A.sqrt();

            b[0] = A * ((A + n1) + (A - n1) * cos_ω0 + n2 * A_sqrt * α);
            b[1] = n2 * A * ((A - n1) + (A + n1) * cos_ω0);
            b[2] = A * ((A + n1) + (A - n1) * cos_ω0 - n2 * A_sqrt * α);

            a[0] = (A + n1) - (A - n1) * cos_ω0 + n2 * A_sqrt * α;
            a[1] = -n2 * ((A - n1) - (A + n1) * cos_ω0);
            a[2] = (A + n1) - (A - n1) * cos_ω0 - n2 * A_sqrt * α;
        }
        FilterType::None => {
            b[0] = n1;
            a[0] = n1;
        }
    }

    // normalize coeffecients
    let a0 = a[0];

    FilterCoeffs {
        b: [b[0] / a0, b[1] / a0, b[2] / a0],
        a: [n1, a[1] / a0, a[2] / a0],
    }
}

#[derive(Debug, Clone)]
pub struct RecursiveFilter<const N: usize, F: Float> {
    coeffs: FilterCoeffs<N, F>,
    /// x[0] = x[n], x[1] = x[n - 1], ...
    x: [F; N],
    /// y[0] = y[n], y[1] = y[n - 1], ...
    y: [F; N],
}

impl<const N: usize, F: Float> RecursiveFilter<N, F> {
    pub fn set_coeffs(&mut self, coeffs: FilterCoeffs<N, F>) {
        self.coeffs = coeffs;
    }

    pub fn reset_history(&mut self) {
        self.x = [F::zero(); N];
        self.y = [F::zero(); N];
    }
}

impl<const N: usize, F: Float> From<FilterCoeffs<N, F>> for RecursiveFilter<N, F> {
    fn from(value: FilterCoeffs<N, F>) -> Self {
        RecursiveFilter {
            coeffs: value,
            x: [F::zero(); N],
            y: [F::zero(); N],
        }
    }
}

impl<const N: usize, F: Float + Sum> RecursiveFilter<N, F> {
    pub fn filter_sample(&mut self, sample: F) -> F {
        let RecursiveFilter {
            coeffs: FilterCoeffs { ref b, ref a },
            x,
            y,
        } = self;

        x[0] = sample;

        let y_n = x.iter().zip(b).map(|(x_n, b_n)| *x_n * *b_n).sum::<F>()
            - y.iter().zip(a).skip(1).map(|(y_n, a_n)| *y_n * *a_n).sum::<F>();

        y[0] = y_n;

        for i in (0..N - 1).rev() {
            x[i + 1] = x[i];
            y[i + 1] = y[i];
        }

        y_n
    }
}

impl<F: Float> RecursiveFilter<3, F> {
    pub fn new(filter_spec: FilterSpec<F>) -> RecursiveFilter<3, F> {
        RecursiveFilter {
            coeffs: filter_coeffs(filter_spec),
            x: [F::zero(); 3],
            y: [F::zero(); 3],
        }
    }

    pub fn set(&mut self, filter_spec: FilterSpec<F>) {
        self.coeffs = filter_coeffs(filter_spec);
    }
}

impl Default for RecursiveFilter<3, f32> {
    fn default() -> Self {
        BiquadFilter::new(FilterSpec::none())
    }
}

pub type BiquadFilter = RecursiveFilter<3, f32>;

// TODO: optimize, this is very sloppy
#[derive(Debug, Clone)]
pub struct NthBiquadFilter<const N: usize> {
    filters: [BiquadFilter; N],
    spec: FilterSpec<f32>,
}

impl<const N: usize> NthBiquadFilter<N> {
    pub fn new(mut spec: FilterSpec<f32>) -> Self {
        spec.filter_type = match spec.filter_type {
            FilterType::Peaking { bandwidth, db_gain } => FilterType::Peaking {
                bandwidth,
                db_gain: db_gain / N as f32,
            },
            FilterType::LowShelf { slope, db_gain } => FilterType::LowShelf {
                slope,
                db_gain: db_gain / N as f32,
            },
            FilterType::HighShelf { slope, db_gain } => FilterType::HighShelf {
                slope,
                db_gain: db_gain / N as f32,
            },
            _ => spec.filter_type,
        };

        let coeffs = filter_coeffs(spec.clone());

        let filters = from_fn(|_| BiquadFilter::from(coeffs.clone()));

        NthBiquadFilter { filters, spec }
    }

    pub fn empty() -> NthBiquadFilter<N> {
        Self::new(FilterSpec {
            f0: 0.5,
            fs: 1.0,
            filter_type: FilterType::LowPass { q: 0.7 },
        })
    }

    pub fn get_order_multiplier(&self) -> usize {
        N
    }

    pub fn get_spec(&self) -> &FilterSpec<f32> {
        &self.spec
    }

    pub fn set_spec(&mut self, mut spec: FilterSpec<f32>) {
        spec.filter_type = match spec.filter_type {
            FilterType::Peaking { bandwidth, db_gain } => FilterType::Peaking {
                bandwidth,
                db_gain: db_gain / N as f32,
            },
            FilterType::LowShelf { slope, db_gain } => FilterType::LowShelf {
                slope,
                db_gain: db_gain / N as f32,
            },
            FilterType::HighShelf { slope, db_gain } => FilterType::HighShelf {
                slope,
                db_gain: db_gain / N as f32,
            },
            _ => spec.filter_type,
        };

        let coeffs = filter_coeffs(spec);

        for filter in &mut self.filters {
            filter.set_coeffs(coeffs.clone());
        }
    }

    pub fn filter_sample(&mut self, x_n: f32) -> f32 {
        let mut intermediate = x_n;

        for filter in &mut self.filters {
            intermediate = filter.filter_sample(intermediate);
        }

        intermediate
    }
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Default)]
pub struct SimpleComb {
    pub M: f32,
    pub α: f32,
}

impl SimpleComb {
    pub fn new(f0: f32, fs: f32, α: f32) -> SimpleComb {
        SimpleComb { M: fs / f0, α }
    }

    #[inline]
    pub fn filter(&self, x: f32, sample: &[f32], position: f32) -> f32 {
        if position < self.M {
            0.0
        } else {
            let tap_pos = position - self.M;

            x + lerp(sample[tap_pos as usize], sample[tap_pos as usize + 1], tap_pos.fract()) * self.α
        }
    }

    // TODO: very sloppy, I don't feel like working out the math though, lol
    pub fn response(&self, freq: f32, fs: f32) -> f32 {
        let ω = 2.0 * PI * (freq / fs);

        let real = 1.0 + self.α * (ω * self.M).cos();
        let imag = self.α * (ω * self.M).sin();

        f32::sqrt(real * real + imag * imag)
    }
}
