use num::Float;
use std::f32::consts::PI;

// (elephant paper) http://yehar.com/blog/wp-content/uploads/2009/08/deip.pdf
// https://stackoverflow.com/questions/1125666/how-do-you-do-bicubic-or-other-non-linear-interpolation-of-re-sampled-audio-da
#[inline]
pub fn hermite_interpolate(x0: f32, x1: f32, x2: f32, x3: f32, t: f32) -> f32 {
    let diff = x1 - x2;
    let c1 = x2 - x0;
    let c3 = x3 - x0 + 3.0 * diff;
    let c2 = -(2.0 * diff + c1 + c3);

    0.5 * ((c3 * t + c2) * t + c1) * t + x1
}

#[inline]
pub fn lerp<F: Float>(start: F, end: F, amount: F) -> F {
    (end - start) * amount + start
}

#[inline]
pub fn hermite_lookup(sample: &[f32], position: f32) -> f32 {
    let pos_usize = position as usize;

    let x_minus_1 = if pos_usize > 0 { sample[pos_usize - 1] } else { 0.0 };

    hermite_interpolate(
        x_minus_1,
        sample.get(pos_usize).copied().unwrap_or(0.0),
        sample.get(pos_usize + 1).copied().unwrap_or(0.0),
        sample.get(pos_usize + 2).copied().unwrap_or(0.0),
        position.fract(),
    )
}

#[inline]
pub fn lerp_lookup(sample: &[f32], position: f32) -> f32 {
    let pos_usize = position as usize;

    lerp(
        sample.get(pos_usize).copied().unwrap_or(0.0),
        sample.get(pos_usize + 1).copied().unwrap_or(0.0),
        position.fract(),
    )
}

pub fn cos_erp(start: f32, end: f32, x: f32) -> f32 {
    f32::cos(x * PI / 2.0) * start + f32::cos((1.0 - x) * PI / 2.0) * end
}
