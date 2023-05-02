use crate::{node::biquad_filter::BiquadFilter, util::lerp};

use super::tables::{BASE_FREQUENCY, WAVETABLE_MASK, WAVETABLE_SIZE};

#[inline]
pub fn interpolate_osc(wavetable: &[[f32; WAVETABLE_SIZE]], frequency: f32, phase: f32) -> f32 {
    let phase = phase - phase.floor(); // make phase bound

    let wavetable_index = (frequency / BASE_FREQUENCY) as usize; // which wavetable to use (rounded down)
    let sample_index = (phase * WAVETABLE_SIZE as f32) as usize; // which sample
    let sample_offset = phase * WAVETABLE_SIZE as f32; // interpolate between samples
    let sample_offset = sample_offset - sample_offset.floor();

    let lower_old = wavetable[wavetable_index][sample_index];
    let lower_new = wavetable[wavetable_index][(sample_index + 1) & WAVETABLE_MASK];

    let upper_old = wavetable[wavetable_index + 1][sample_index];
    let upper_new = wavetable[wavetable_index + 1][(sample_index + 1) & WAVETABLE_MASK];

    let sample_lower = lerp(lower_old, lower_new, sample_offset);
    let sample_higher = lerp(upper_old, upper_new, sample_offset);

    lerp(
        sample_lower,
        sample_higher,
        (frequency - (BASE_FREQUENCY * (wavetable_index) as f32)) / BASE_FREQUENCY,
    )
}

#[inline]
pub fn interpolate(wavetable: &[f32], filter: &mut BiquadFilter, phase: f32) -> f32 {
    filter.reset_history();

    let phase = phase - phase.floor(); // make phase bound

    let wavetable_width = wavetable.len();

    let sample_index = (phase * wavetable_width as f32) as i32; // which sample

    let sample_offset = phase * wavetable_width as f32; // interpolate between samples
    let sample_offset = sample_offset - sample_offset.floor(); // fast % 1

    filter.filter_audio(wavetable[(sample_index - 2).rem_euclid(wavetable_width as i32) as usize]);
    filter.filter_audio(wavetable[(sample_index - 1).rem_euclid(wavetable_width as i32) as usize]);
    let here = filter.filter_audio(wavetable[sample_index as usize]);
    let next = filter.filter_audio(wavetable[(sample_index + 1).rem_euclid(wavetable_width as i32) as usize]);

    lerp(here, next, sample_offset)
}
