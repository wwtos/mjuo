use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub enum ShelfFilterType {
    HighShelf,
    LowShelf,
}

#[derive(Debug, Clone)]
pub struct ShelfFilter {
    /// x[0] = x[n], x[1] = x[n - 1], ...
    x: [f32; 3],
    /// y[0] = y[n], y[1] = y[n - 1], ...
    y: [f32; 3],
    b: [f32; 3],
    a: [f32; 3],
    cos_ω0: f32,
    db_gain: f32,
    α: f32,
    filter_type: ShelfFilterType,
}

const SHELF_SLOPE: f32 = 1.0;

/// maximum Q before ripples appear
#[allow(non_snake_case)]
pub fn max_shelf_q(db_gain: f32) -> f32 {
    let A = 10_f32.powf(db_gain / 40.0);

    let one_over_Q = f32::sqrt((A + 1.0 / A) * (1.0 / SHELF_SLOPE - 1.0) + 2.0);

    1.0 / one_over_Q
}

// from audio EQ cookbook, as direct as possible (https://www.w3.org/TR/audio-eq-cookbook/)
#[allow(non_snake_case)]
impl ShelfFilter {
    pub fn new(fs: f32, filter_type: ShelfFilterType, f0: f32, Q: f32, db_gain: f32) -> Self {
        // equivalent to `(10 ** (db_gain / 20)).sqrt()`, aka sqrt of amplitude
        let A = 10_f32.powf(db_gain / 40.0);
        let A_sqrt = A.sqrt();

        let ω0 = 2.0 * PI * f0 / fs;
        let sin_ω0 = ω0.sin();
        let cos_ω0 = ω0.cos();

        let α = sin_ω0 / 2.0 * Q;

        let mut b = [0_f32; 3];
        let mut a = [0_f32; 3];

        match filter_type {
            ShelfFilterType::LowShelf => {
                b[0] = A * ((A + 1.0) - (A - 1.0) * cos_ω0 + 2.0 * A_sqrt * α);
                b[1] = 2.0 * A * ((A - 1.0) - (A + 1.0) * cos_ω0);
                b[2] = A * ((A + 1.0) - (A - 1.0) * cos_ω0 - 2.0 * A_sqrt * α);

                a[0] = (A + 1.0) + (A - 1.0) * cos_ω0 + 2.0 * A_sqrt * α;
                a[1] = -2.0 * ((A - 1.0) + (A + 1.0) * cos_ω0);
                a[2] = (A + 1.0) + (A - 1.0) * cos_ω0 - 2.0 * A_sqrt * α;
            }
            ShelfFilterType::HighShelf => {
                b[0] = A * ((A + 1.0) + (A - 1.0) * cos_ω0 + 2.0 * A_sqrt * α);
                b[1] = 2.0 * A * ((A - 1.0) + (A + 1.0) * cos_ω0);
                b[2] = A * ((A + 1.0) + (A - 1.0) * cos_ω0 - 2.0 * A_sqrt * α);

                a[0] = (A + 1.0) - (A - 1.0) * cos_ω0 + 2.0 * A_sqrt * α;
                a[1] = -2.0 * ((A - 1.0) - (A + 1.0) * cos_ω0);
                a[2] = (A + 1.0) - (A - 1.0) * cos_ω0 - 2.0 * A_sqrt * α;
            }
        }

        // normalize coeffecients
        let a0 = a[0];

        b[0] /= a0;
        b[1] /= a0;
        b[2] /= a0;

        a[0] = 1.0;
        a[1] /= a0;
        a[2] /= a0;

        ShelfFilter {
            x: [0.0; 3],
            y: [0.0; 3],
            b,
            a,
            α,
            cos_ω0,
            db_gain,
            filter_type,
        }
    }

    pub fn empty() -> ShelfFilter {
        ShelfFilter {
            x: [0.0; 3],
            y: [0.0; 3],
            b: [1.0, 0.0, 0.0],
            a: [0.0; 3],
            α: 0.0,
            cos_ω0: 0.0,
            db_gain: 0.0,
            filter_type: ShelfFilterType::HighShelf,
        }
    }

    pub fn get_db_gain(&self) -> f32 {
        self.db_gain
    }

    pub fn set_db_gain(&mut self, db_gain: f32) {
        self.db_gain = db_gain;

        let A = 10_f32.powf(db_gain / 40.0);
        let A_sqrt = A.sqrt();

        let ShelfFilter {
            ref mut b,
            ref mut a,
            ref cos_ω0,
            ref α,
            filter_type,
            ..
        } = self;

        match filter_type {
            ShelfFilterType::LowShelf => {
                b[0] = A * ((A + 1.0) - (A - 1.0) * cos_ω0 + 2.0 * A_sqrt * α);
                b[1] = 2.0 * A * ((A - 1.0) - (A + 1.0) * cos_ω0);
                b[2] = A * ((A + 1.0) - (A - 1.0) * cos_ω0 - 2.0 * A_sqrt * α);

                a[0] = (A + 1.0) + (A - 1.0) * cos_ω0 + 2.0 * A_sqrt * α;
                a[1] = -2.0 * ((A - 1.0) + (A + 1.0) * cos_ω0);
                a[2] = (A + 1.0) + (A - 1.0) * cos_ω0 - 2.0 * A_sqrt * α;
            }
            ShelfFilterType::HighShelf => {
                b[0] = A * ((A + 1.0) + (A - 1.0) * cos_ω0 + 2.0 * A_sqrt * α);
                b[1] = 2.0 * A * ((A - 1.0) + (A + 1.0) * cos_ω0);
                b[2] = A * ((A + 1.0) + (A - 1.0) * cos_ω0 - 2.0 * A_sqrt * α);

                a[0] = (A + 1.0) - (A - 1.0) * cos_ω0 + 2.0 * A_sqrt * α;
                a[1] = -2.0 * ((A - 1.0) - (A + 1.0) * cos_ω0);
                a[2] = (A + 1.0) - (A - 1.0) * cos_ω0 - 2.0 * A_sqrt * α;
            }
        }

        // normalize coeffecients
        let a0 = a[0];

        b[0] /= a0;
        b[1] /= a0;
        b[2] /= a0;

        a[0] = 1.0;
        a[1] /= a0;
        a[2] /= a0;
    }

    pub fn filter_sample(&mut self, x_n: f32) -> f32 {
        let ShelfFilter { mut x, mut y, a, b, .. } = self;

        x[0] = x_n;

        #[rustfmt::skip]
        let y_n = b[0] * x[0] + b[1] * x[1] + b[2] * x[2] -
                                a[1] * y[1] - a[2] * y[2];

        y[0] = y_n;

        // shift values
        x[2] = x[1];
        x[1] = x[0];

        y[2] = y[1];
        y[1] = y[0];

        y_n
    }
}
