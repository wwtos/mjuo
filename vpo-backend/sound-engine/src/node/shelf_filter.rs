#[derive(Debug, Clone)]
pub enum ShelfFilterType {
    HighShelf,
    LowShelf,
}

#[derive(Debug, Clone)]
pub struct ShelfFilter {
    prev_input_1: f32,
    prev_input_2: f32,
    prev_output_1: f32,
    prev_output_2: f32,
    b: [f32; 3],
    a: [f32; 3],
    w_sin: f32,
    w_cos: f32,
    beta: f32,
    filter_type: ShelfFilterType,
}

// https://github.com/jatinchowdhury18/audio_dspy/blob/master/audio_dspy/eq_design.py
impl ShelfFilter {
    pub fn new(filter_type: ShelfFilterType, fc: f32, Q: f32, gain: f32, fs: f32) -> Self {
        let A = gain.sqrt();
        let wc = 2.0 * std::f32::consts::PI * fc / fs;
        let w_sin = wc.sin();
        let w_cos = wc.cos();
        let beta = A.sqrt() / Q;

        let a0 = (A + 1.0) - ((A - 1.0) * w_cos) + (beta * w_sin);

        let mut b = [0_f32; 3];
        let mut a = [0_f32; 3];

        match filter_type {
            ShelfFilterType::HighShelf => {
                b[0] = A * ((A + 1.0) + ((A - 1.0) * w_cos) + (beta * w_sin)) / a0;
                b[1] = -2.0 * A * ((A - 1.0) + ((A + 1.0) * w_cos)) / a0;
                b[2] = A * ((A + 1.0) + ((A - 1.0) * w_cos) - (beta * w_sin)) / a0;

                a[0] = 1.0;
                a[1] = 2.0 * ((A - 1.0) - ((A + 1.0) * w_cos)) / a0;
                a[2] = ((A + 1.0) - ((A - 1.0) * w_cos) - (beta * w_sin)) / a0;
            }
            ShelfFilterType::LowShelf => {
                b[0] = A * ((A + 1.0) - ((A - 1.0) * w_cos) + (beta * w_sin)) / a0;
                b[1] = -2.0 * A * ((A - 1.0) - ((A + 1.0) * w_cos)) / a0;
                b[2] = A * ((A + 1.0) - ((A - 1.0) * w_cos) - (beta * w_sin)) / a0;

                a[0] = 1.0;
                a[1] = 2.0 * ((A - 1.0) + ((A + 1.0) * w_cos)) / a0;
                a[2] = ((A + 1.0) + ((A - 1.0) * w_cos) - (beta * w_sin)) / a0;
            }
        }

        ShelfFilter {
            prev_input_1: 0.0,
            prev_input_2: 0.0,
            prev_output_1: 0.0,
            prev_output_2: 0.0,
            b,
            a,
            w_sin,
            w_cos,
            beta,
            filter_type,
        }
    }

    pub fn empty() -> ShelfFilter {
        ShelfFilter {
            prev_input_1: 0.0,
            prev_input_2: 0.0,
            prev_output_1: 0.0,
            prev_output_2: 0.0,
            b: [1.0, 0.0, 0.0],
            a: [0.0; 3],
            w_sin: 0.0,
            w_cos: 0.0,
            beta: 0.0,
            filter_type: ShelfFilterType::HighShelf,
        }
    }

    pub fn set_gain(&mut self, gain: f32) {
        let A = gain.sqrt();
        let a0 = (A + 1.0) - ((A - 1.0) * self.w_cos) + (self.beta * self.w_sin);

        let ShelfFilter {
            ref mut b,
            ref mut a,
            ref w_sin,
            ref w_cos,
            ref beta,
            filter_type,
            ..
        } = self;

        match filter_type {
            ShelfFilterType::HighShelf => {
                b[0] = A * ((A + 1.0) + ((A - 1.0) * w_cos) + (beta * w_sin)) / a0;
                b[1] = -2.0 * A * ((A - 1.0) + ((A + 1.0) * w_cos)) / a0;
                b[2] = A * ((A + 1.0) + ((A - 1.0) * w_cos) - (beta * w_sin)) / a0;

                a[0] = 1.0;
                a[1] = 2.0 * ((A - 1.0) - ((A + 1.0) * w_cos)) / a0;
                a[2] = ((A + 1.0) - ((A - 1.0) * w_cos) - (beta * w_sin)) / a0;
            }
            ShelfFilterType::LowShelf => {
                b[0] = A * ((A + 1.0) - ((A - 1.0) * w_cos) + (beta * w_sin)) / a0;
                b[1] = -2.0 * A * ((A - 1.0) - ((A + 1.0) * w_cos)) / a0;
                b[2] = A * ((A + 1.0) - ((A - 1.0) * w_cos) - (beta * w_sin)) / a0;

                a[0] = 1.0;
                a[1] = 2.0 * ((A - 1.0) + ((A + 1.0) * w_cos)) / a0;
                a[2] = ((A + 1.0) + ((A - 1.0) * w_cos) - (beta * w_sin)) / a0;
            }
        }
    }

    pub fn filter_sample(&mut self, x: f32) -> f32 {
        #[rustfmt::skip]
        let output = (self.b[0] * x) +
                     (self.b[1] * self.prev_input_1) +
                     (self.b[2] * self.prev_input_2) -
                     (self.a[1] * self.prev_output_1) -
                     (self.a[2] * self.prev_output_2);

        self.prev_input_2 = self.prev_input_1;
        self.prev_input_1 = x;

        self.prev_output_2 = self.prev_output_1;
        self.prev_output_1 = output;

        output
    }
}
