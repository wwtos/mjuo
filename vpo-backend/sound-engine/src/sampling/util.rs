use nalgebra::DVector;

pub fn sign(num: f64) -> f64 {
    if num > 0.0 {
        1.0
    } else if num < 0.0 {
        -1.0
    } else {
        0.0
    }
}

pub fn sq(x: f64) -> f64 {
    x * x
}

pub fn mean(x: &[f64]) -> f64 {
    x.iter().sum::<f64>() / x.len() as f64
}

pub fn std(x: &[f64]) -> f64 {
    let mean = mean(x);

    let squared_diff = f64::sqrt(x.iter().map(|x| sq(x - mean)).sum::<f64>() / x.len() as f64);

    f64::sqrt(squared_diff / (x.len() - 1) as f64)
}

pub fn rms(x: &[f64]) -> f64 {
    let squared_diff: f64 = x.iter().map(|x| sq(*x)).sum();

    f64::sqrt(squared_diff / (x.len() - 1) as f64)
}

pub fn argmin(x: &[f64]) -> Option<usize> {
    x.iter()
        .enumerate()
        .fold((f64::INFINITY, None), |(min_so_far, out), (i, val)| {
            if val < &min_so_far {
                (*val, Some(i))
            } else {
                (min_so_far, out)
            }
        })
        .1
}

pub fn argmax(x: &[f64]) -> Option<usize> {
    x.iter()
        .enumerate()
        .fold((-f64::INFINITY, None), |(max_so_far, out), (i, val)| {
            if val > &max_so_far {
                (*val, Some(i))
            } else {
                (max_so_far, out)
            }
        })
        .1
}

pub fn gradient(func: &[f64]) -> Vec<f64> {
    let mut res = Vec::with_capacity(func.len());
    res.push(0.0);

    for i in 1..(func.len() - 1) {
        res.push((func[i + 1] - func[i - 1]) / 2.0);
    }

    res.push(0.0);
    return res;
}

// (elephant paper) http://yehar.com/blog/wp-content/uploads/2009/08/deip.pdf
// https://stackoverflow.com/questions/1125666/how-do-you-do-bicubic-or-other-non-linear-interpolation-of-re-sampled-audio-da
pub fn hermite_interpolate(x0: f64, x1: f64, x2: f64, x3: f64, t: f64) -> f64 {
    let c0 = x1;
    let c1 = 0.5 * (x2 - x0);
    let c2 = x0 - (2.5 * x1) + (2.0 * x2) - (0.5 * x3);
    let c3 = (0.5 * (x3 - x0)) + (1.5 * (x1 - x2));

    (((((c3 * t) + c2) * t) + c1) * t) + c0
}

pub fn resample_to(sig: &[f64], new_length: usize) -> Vec<f64> {
    // figure out the difference in length
    let step_by = sig.len() as f64 / new_length as f64;

    let mut new_sig = Vec::with_capacity(new_length);

    for i in 0..new_length {
        let pos_in_sig = i as f64 * step_by;
        let pos_in_sig_slice = pos_in_sig as i64;

        if pos_in_sig_slice == 0 || pos_in_sig_slice >= sig.len() as i64 - 2 {
            new_sig.push(sig[pos_in_sig as usize]);
            continue;
        }

        let pos_in_sig_slice = (pos_in_sig_slice - 1) as usize;

        new_sig.push(hermite_interpolate(
            sig[pos_in_sig_slice],
            sig[pos_in_sig_slice + 1],
            sig[pos_in_sig_slice + 2],
            sig[pos_in_sig_slice + 3],
            pos_in_sig % 1.0,
        ));
    }

    new_sig
}

pub fn norm_signal(signal: &DVector<f64>) -> (DVector<f64>, f64, f64) {
    let signal_min = signal.min();
    let signal_max = signal.max();

    let signal_norm = (((signal.add_scalar(-signal_min)) / (signal_max - signal_min)).scale(2.0)).add_scalar(-1.0);

    (signal_norm, signal_min, signal_max)
}
