use std::path::PathBuf;

use interp::interp;
use nalgebra::DVector;
use resource_manager::Resource;

use crate::{
    node::envelope,
    sampling::util::{resample_to_lin, sq},
};

use super::{
    sample::Sample,
    savitzky_golay::savgol_filter,
    util::{
        abs, argmax, argmin, gradient, hann, hermite_interpolate, mean, median, mult, norm_signal, resample_to, s_sub,
        sign, std,
    },
};

struct EnvelopePoints {
    attack: usize,
    loop_end: usize,
    release: usize,
}

// https://stackoverflow.com/questions/34235530/how-to-get-high-and-low-envelope-of-a-signal
fn envelopes_idx(signal: &DVector<f64>, dmax: usize) -> Vec<usize> {
    // locals min
    let mut lmax: Vec<usize> = Vec::new();

    let mut last_sign = 0.0;
    for i in 1..signal.len() {
        let sign = sign(signal[i] - signal[i - 1]);

        if last_sign - sign > 0.0 {
            lmax.push(i);
        }

        last_sign = sign;
    }

    let mut chunked_lmax: Vec<usize> = Vec::new();
    for (i, local_maximum) in lmax.chunks(dmax).enumerate() {
        let mapped_chunk = DVector::from_iterator(local_maximum.len(), local_maximum.iter().map(|pos| signal[*pos]));
        let local_maximum_pos = lmax[i * dmax + mapped_chunk.argmax().0];

        chunked_lmax.push(local_maximum_pos);
    }

    chunked_lmax
}

pub fn calc_amp(signal: &[f64], dmax: usize, sample_rate: u32) -> Vec<f64> {
    let mut env_idx = envelopes_idx(&DVector::from_row_slice(signal), dmax);
    env_idx.insert(0, 0);
    env_idx.push(signal.len() - 1);

    let env_idx_as_f64: Vec<f64> = env_idx.iter().map(|x| *x as f64).collect();
    let env_points: Vec<f64> = env_idx.iter().map(|idx| signal[*idx]).collect();

    let interp: Vec<f64> = (0..signal.len())
        .step_by(100)
        .map(|x| interp(&env_idx_as_f64, &env_points, x as f64))
        .collect();
    let interp_smoothed = savgol_filter(&interp, sample_rate / 1500, 5, 2);

    resample_to(&interp, signal.len())
}

struct SearchSettings {
    pub search_width: usize,
    pub search_step: usize,
    pub peak_attack: usize,
    pub peak_release: usize,
    pub too_far_in_percentage_attack: f64,
    pub too_far_in_percentage_release: f64,
}

fn search_for_attack(envelope: &[f64], envelope_deriv: &[f64], settings: &SearchSettings) -> usize {
    let SearchSettings {
        search_width,
        search_step,
        peak_attack,
        peak_release,
        too_far_in_percentage_attack,
        too_far_in_percentage_release: _,
    } = settings;

    let search_start = *peak_attack;
    let search_end = ((envelope.len() as f64 * too_far_in_percentage_attack) as usize).min(peak_release - search_width);
    let search_span = search_end - search_start;

    let env_deriv_median = median(envelope_deriv);

    let window = hann(*search_width);

    let attack_index = (search_start..search_end)
        .step_by(*search_step)
        .map(|i| {
            let env_slice = mult(&envelope_deriv[i..(i + search_width)], &window);
            let env_slice_mean = mean(&env_slice);

            let median_dist = (env_slice_mean - env_deriv_median).abs();

            (median_dist, i + search_width / 2)
        })
        .min_by(|x, y| x.0.total_cmp(&y.0))
        .unwrap()
        .1;

    attack_index
}

fn search_for_release(envelope: &[f64], envelope_deriv: &[f64], settings: &SearchSettings) -> usize {
    let SearchSettings {
        search_width,
        search_step,
        peak_attack,
        peak_release,
        too_far_in_percentage_attack,
        too_far_in_percentage_release: _,
    } = settings;

    let search_start =
        ((envelope_deriv.len() as f64 * too_far_in_percentage_attack) as usize).min(peak_release - search_width);
    let search_end = peak_release - search_width;
    let search_span = search_end - search_start;

    let env_deriv_median = median(envelope_deriv);
    let env_deriv_std = std(envelope_deriv);

    let window = &hann(*search_width * 2)[*search_width..(search_width * 2)];

    println!("std: {}", env_deriv_std);

    let mut release_indexes = (search_start..search_end)
        .step_by(*search_step)
        .map(|i| {
            let env_slice = &mult(&envelope_deriv[i..(i + search_width)], window);
            let env_slice_mean = mean(env_slice);

            let median_dist = (env_slice_mean - env_deriv_median).abs() / env_deriv_std;

            let beginning_penalty = 0.3 * sq(1.0 - ((i - search_start) as f64) / search_span as f64);

            (median_dist + beginning_penalty, i + search_width)
        })
        .collect::<Vec<(f64, usize)>>();

    release_indexes.sort_by(|a, b| a.0.total_cmp(&b.0));

    println!("top picks: {:?}", &release_indexes[0..5]);

    release_indexes[0].1
}

pub fn find_envelope(sample: &[f64], freq: f64, sample_rate: u32) /* -> EnvelopePoints*/
{
    let (sample_norm, sample_min, sample_max) = norm_signal(&DVector::from_row_slice(sample));

    let envelope = calc_amp(
        sample_norm.as_slice(),
        ((sample_rate as f64 / freq) * 2.0) as usize,
        sample_rate,
    );

    // find min envelope value and shift values, so we don't get NaN from log10
    let envelope_min = -envelope.clone().into_iter().reduce(f64::min).unwrap() + 0.2;
    let envelope_db: Vec<f64> = envelope.iter().map(|x| (x + envelope_min).log10() * 20.0).collect();

    let envelope_deriv = gradient(&envelope_db);
    println!("deriv: {:?}", &resample_to_lin(&envelope_deriv, 400));

    let env_deriv_mean = mean(&envelope_deriv);
    let env_deriv_std = std(&envelope_deriv);

    let peak_attack = argmax(&envelope_deriv).unwrap();
    let peak_release = argmin(&envelope_deriv).unwrap();

    let search_settings = SearchSettings {
        search_width: 5000,
        search_step: 50,
        peak_attack,
        peak_release,
        too_far_in_percentage_attack: 0.2,
        too_far_in_percentage_release: 0.7,
    };

    let attack_index = search_for_attack(&envelope, &envelope_deriv, &search_settings);
    let release_index = search_for_release(&envelope, &envelope_deriv, &search_settings);

    let env_deriv_median = median(&envelope_deriv);

    println!(
        "attack index: {}, release_index: {}, release_deriv: {}, median: {}",
        attack_index,
        release_index,
        (envelope_deriv[release_index] - env_deriv_median).abs(),
        env_deriv_median,
    );
}

#[test]
fn envelopes_idx_test() {
    let foo = Sample::load_resource(&PathBuf::from("/home/mason/rust/mjuo/vpo-backend/060-C.wav")).unwrap();

    let audio = foo.buffer.audio_raw;

    let audio_f64: Vec<f64> = audio.iter().map(|x| *x as f64).collect();
    let amp = find_envelope(&audio_f64, 261.63, foo.buffer.sample_rate);

    // println!("{:?}", resample_to(&amp, 400));

    //envelopes_idx(&DVector::from(test_sig), 10);
}
