// https://github.com/arntanguy/gram_savitzky_golay/blob/master/src/gram_savitzky_golay.cpp
fn gram_poly(i: i32, m: i32, k: i32, s: i32) -> f64 {
    if k > 0 {
        (4.0 * k as f64 - 2.0) / (k as f64 * (2.0 * m as f64 - k as f64 + 1.0))
            * (i as f64 * gram_poly(i, m, k - 1, s) + s as f64 * gram_poly(i, m, k - 1, s - 1))
            - ((k as f64 - 1.0) * (2.0 * m as f64 + k as f64)) / (k as f64 * (2.0 * m as f64 - k as f64 + 1.0))
                * gram_poly(i, m, k - 2, s)
    } else {
        if k == 0 && s == 0 {
            1.0
        } else {
            0.0
        }
    }
}

fn gen_fact(a: i32, b: i32) -> f64 {
    let mut gf: f64 = 1.0;

    for j in ((a - b) + 1)..=a {
        gf *= j as f64;
    }

    return gf;
}

fn weight(i: i32, t: i32, m: i32, n: i32, s: i32) -> f64 {
    let mut w: f64 = 0.0;

    for k in 0..=n {
        w = w
            + (2.0 * k as f64 + 1.0)
                * (gen_fact(2 * m as i32, k) / gen_fact(2 * m as i32 + k + 1, k + 1))
                * gram_poly(i, m, k, 0)
                * gram_poly(t, m, k, s);
    }

    w
}

fn compute_weights(m: i32, t: i32, n: i32, s: i32) -> Vec<f64> {
    let mut weights: Vec<f64> = Vec::with_capacity(2 * m as usize + 1);

    for i in 0..(2 * m + 1) {
        weights.push(weight(i - m, t, m, n, s));
    }

    weights
}

pub struct SavitzkyGolayFilter {
    dt: f64,
    weights: Vec<f64>,
}

impl SavitzkyGolayFilter {
    pub fn new(m: u32, t: u32, n: i32, s: i32, dt: f64) -> SavitzkyGolayFilter {
        let weights = compute_weights(m as i32, t as i32, n, s);
        let dt = dt.powf(s as f64);

        SavitzkyGolayFilter { dt, weights }
    }

    pub fn filter(&self, sig: &[f64]) -> f64 {
        let mut res = self.weights[0] * sig[0];

        for i in 1..sig.len() {
            res += self.weights[i] * sig[i];
        }

        res / self.dt
    }
}

pub fn savgol_filter(sig: &Vec<f64>, window_length: u32, t: u32, polyorder: i32) -> Vec<f64> {
    let filter = SavitzkyGolayFilter::new(window_length, t, polyorder, 0, 1.0);

    (0..sig.len())
        .map(|i| {
            let chunk_start = i;
            let chunk_end = (i + window_length as usize * 2 + 1).min(sig.len());

            let chunk = &sig[chunk_start..chunk_end];

            filter.filter(chunk)
        })
        .collect()
}
