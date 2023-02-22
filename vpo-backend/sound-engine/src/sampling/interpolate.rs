use std::f32::consts::PI;

// (elephant paper) http://yehar.com/blog/wp-content/uploads/2009/08/deip.pdf
// https://stackoverflow.com/questions/1125666/how-do-you-do-bicubic-or-other-non-linear-interpolation-of-re-sampled-audio-da
pub fn hermite_interpolate(x0: f32, x1: f32, x2: f32, x3: f32, t: f32) -> f32 {
    let c0 = x1;
    let c1 = 0.5 * (x2 - x0);
    let c2 = x0 - (2.5 * x1) + (2.0 * x2) - (0.5 * x3);
    let c3 = (0.5 * (x3 - x0)) + (1.5 * (x1 - x2));

    (((((c3 * t) + c2) * t) + c1) * t) + c0
}

pub fn lerp(start: f32, end: f32, amount: f32) -> f32 {
    (end - start) * amount + start
}

pub fn cos_erp(start: f32, end: f32, x: f32) -> f32 {
    f32::cos(x * PI / 2.0) * start + f32::cos((1.0 - x) * PI / 2.0) * end
}
