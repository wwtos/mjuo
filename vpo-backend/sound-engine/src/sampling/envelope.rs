#[cfg(test)]
use {super::sample::Sample, resource_manager::Resource, std::path::PathBuf};

use std::{fs::File, io::Write};

use interp::interp;
use nalgebra::DVector;
use pitch_detection::detector::{mcleod::McLeodDetector, PitchDetector};
use serde_json::json;

use crate::{
    node::envelope,
    sampling::util::{resample_to_lin, sq},
};

use super::{
    savitzky_golay::savgol_filter,
    util::{abs, argmax, argmin, gradient, hann, mean, median, mult, norm_signal, resample_to, rms, sign, std},
};

pub struct SampleMetadata {
    pub attack_index: usize,
    pub release_index: usize,
    pub loop_start: usize,
    pub loop_end: usize,
    pub note: u8,
    pub cents: i16,
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

pub fn calc_amp(signal: &[f64], dmax: usize, step_by: usize, sample_rate: u32) -> Vec<f64> {
    let mut env_idx = envelopes_idx(&DVector::from_row_slice(signal), dmax);
    env_idx.insert(0, 0);
    env_idx.push(signal.len() - 1);

    let env_idx_as_f64: Vec<f64> = env_idx.iter().map(|x| *x as f64).collect();
    let env_points: Vec<f64> = env_idx.iter().map(|idx| signal[*idx]).collect();

    let interp: Vec<f64> = (0..signal.len())
        .step_by(step_by)
        .map(|x| interp(&env_idx_as_f64, &env_points, x as f64))
        .collect();
    let interp_smoothed = savgol_filter(&interp, sample_rate / 2000, 5, 2);

    resample_to(&interp, signal.len())
}

pub fn calc_amp_2(signal: &[f64], dmax: usize, step_by: usize, sample_rate: u32) -> Vec<f64> {
    (0..(signal.len() - dmax)).map(|i| rms(&signal[i..i + dmax])).collect()
}

struct EnvelopeSettings {
    pub search_width: usize,
    pub search_step: usize,
    pub peak_attack: usize,
    pub peak_release: usize,
    pub end_incentive: f64,
    pub too_far_in_percentage_attack: f64,
    pub too_far_in_percentage_release: f64,
    pub attack_shift: i32,
    pub release_shift: i32,
}

struct LoopSettings {
    pub derivative_threshold: f64,
    pub min_loop_length: f64,
    pub distance_between_loops: f64,
    pub quality_factor: f64,
    pub final_pass_count: usize,
}

fn search_for_attack(envelope_deriv: &[f64], settings: &EnvelopeSettings) -> usize {
    let EnvelopeSettings {
        search_width,
        search_step,
        peak_attack,
        peak_release,
        end_incentive: _,
        too_far_in_percentage_attack,
        too_far_in_percentage_release: _,
        attack_shift,
        release_shift: _,
    } = settings;

    let search_start = *peak_attack;
    let search_end =
        ((envelope_deriv.len() as f64 * too_far_in_percentage_attack) as usize).min(peak_release - search_width);

    let env_deriv_median = median(envelope_deriv);

    let window = &hann(*search_width * 2)[*search_width..(search_width * 2)];

    let attack_index = (search_start..search_end)
        .step_by(*search_step)
        .map(|i| {
            let env_slice = mult(&envelope_deriv[i..(i + search_width)], &window);
            let env_slice_mean = mean(&env_slice);

            let median_dist = (env_slice_mean - env_deriv_median).abs();

            (median_dist, i)
        })
        .min_by(|x, y| x.0.total_cmp(&y.0))
        .unwrap()
        .1;

    (attack_index as i32 + attack_shift) as usize
}

fn search_for_release(sample: &[f64], env_deriv: &[f64], settings: &EnvelopeSettings) -> usize {
    let EnvelopeSettings {
        search_width,
        search_step,
        peak_attack,
        peak_release,
        end_incentive,
        too_far_in_percentage_attack: _,
        too_far_in_percentage_release,
        attack_shift: _,
        release_shift,
    } = settings;

    let search_start =
        ((env_deriv.len() as f64 * too_far_in_percentage_release) as usize).min(peak_release - search_width);

    let env_deriv_std = std(&env_deriv[*peak_attack..*peak_release]);
    let env_deriv_median = median(&env_deriv[*peak_attack..*peak_release]);

    let threshold = (env_deriv[*peak_release] - env_deriv_median).abs() * 0.5;

    let mut outside_of_threshold_last_time = false;
    let mut release_index: usize = 0;

    for i in search_start..*peak_release {
        if (env_deriv[i] - env_deriv_median).abs() > threshold {
            if !outside_of_threshold_last_time {
                release_index = i;
                outside_of_threshold_last_time = true;
            }
        } else {
            outside_of_threshold_last_time = false;
        }
    }

    println!(
        "release index: {}, threshold: {}, search start: {}, search end: {}, length: {}",
        release_index,
        threshold,
        search_start,
        *peak_release,
        sample.len(),
    );

    release_index = (release_index as i32 + release_shift) as usize;

    // find part in sample close to 0
    let search_area = &sample[release_index..(release_index + 1000)];
    release_index += argmin(&abs(search_area)).unwrap();

    println!("\n\nrelease index: {}\n\n", release_index);

    release_index
}

fn find_loop_point(
    loop_settings: &LoopSettings,
    sample: &[f64],
    freq: f64,
    sample_rate: u32,
    attack_index: usize,
    release_index: usize,
) -> (usize, usize) {
    let LoopSettings {
        derivative_threshold,
        min_loop_length,
        distance_between_loops,
        quality_factor,
        final_pass_count: _,
    } = loop_settings;

    let sample_deriv = gradient(sample);
    let max_derivative = std(&sample_deriv);
    let derivative_threshold = max_derivative * derivative_threshold;

    let slice_width = ((sample_rate as f64 / freq) * 2.0).max(512.0) as usize;
    let min_loop_length = (min_loop_length * sample_rate as f64) as usize;
    let distance_between_loops = (distance_between_loops * sample_rate as f64) as usize;
    let quality_factor = (quality_factor * quality_factor) / 32767.0;

    let indicies_passed: Vec<usize> = sample_deriv
        .iter()
        .enumerate()
        .filter_map(|(i, &val)| {
            if i > attack_index && i < release_index && val.abs() < derivative_threshold {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    let mut found_loops: Vec<((usize, usize), f64)> = Vec::new();
    for from_index in indicies_passed.iter() {
        for to_index in indicies_passed.iter() {
            if *to_index < from_index + min_loop_length {
                continue;
            }

            if from_index + slice_width >= sample.len() || to_index + slice_width >= sample.len() {
                continue;
            }

            if !found_loops.is_empty() && from_index - found_loops.last().unwrap().0 .0 < distance_between_loops {
                continue;
            }

            let cross = (-5..6).fold(0.0, |acc, i| {
                acc + sq(sample[(i + *from_index as i64) as usize] - sample[(i + *to_index as i64) as usize])
            });
            let correlation_value = cross / 10.0;

            if correlation_value < quality_factor {
                found_loops.push(((*from_index, *to_index), correlation_value));
            }
        }
    }

    found_loops.sort_by(|x, y| x.1.total_cmp(&y.1));

    found_loops[0].0
}

pub fn calc_sample_metadata(sample_raw: &[f32], sample_rate: u32, freq: Option<f64>) -> SampleMetadata {
    let sample: Vec<f64> = sample_raw.iter().map(|&x| x as f64).collect();

    let freq = if let Some(freq) = freq {
        freq
    } else {
        let size: usize = sample.len();
        let padding: usize = size / 2;
        let power_threshold: f64 = 5.0;
        let clarity_threshold: f64 = 0.5;

        let mut detector = McLeodDetector::new(size, padding);

        let pitch = detector
            .get_pitch(&sample, sample_rate as usize, power_threshold, clarity_threshold)
            .unwrap();

        pitch.frequency
    };

    let (sample_norm, ..) = norm_signal(&DVector::from_row_slice(&sample));

    let envelope = calc_amp_2(
        &abs(sample_norm.as_slice()),
        ((sample_rate as f64 / freq) * 1.0) as usize,
        ((sample_rate as f64) / freq) as usize * 2,
        sample_rate,
    );

    let envelope_db: Vec<f64> = savgol_filter(
        &envelope.iter().map(|&x| x.log10() * 20.0).collect::<Vec<f64>>(),
        400,
        20,
        2,
    );

    let envelope_deriv = resample_to(
        &savgol_filter(
            &resample_to(
                &gradient(&envelope_db[0..(envelope_db.len() - 801)]),
                envelope.len() / 10,
            ),
            ((freq.log2() - 4.0) * 80.0) as u32,
            20,
            2,
        ),
        envelope.len(),
    );

    let mut f = File::create("/tmp/test.json").expect("Unable to create file");
    f.write_all(serde_json::to_string(&envelope_deriv).unwrap().as_bytes())
        .expect("Unable to write data");

    println!("deriv length: {}", envelope_deriv.len());

    let peak_attack = argmax(&envelope_deriv[0..(envelope_deriv.len() / 2)]).unwrap();
    let possible_peak_release =
        argmin(&envelope_deriv[(envelope_deriv.len() / 2)..envelope_deriv.len()]).unwrap() + envelope_deriv.len() / 2;
    let mut peak_release = possible_peak_release;

    // are there any points after peak_release that reach at least `0.7*envelope_deriv[peak_release]`?
    for i in possible_peak_release..envelope_deriv.len() {
        if envelope_deriv[i] < envelope_deriv[possible_peak_release] * 0.7 {
            peak_release = i;
        }
    }

    let search_settings = EnvelopeSettings {
        search_width: 20000,
        search_step: 1000,
        peak_attack,
        peak_release,
        end_incentive: 0.8,
        too_far_in_percentage_attack: 0.2,
        too_far_in_percentage_release: 0.5,
        attack_shift: 2000,
        release_shift: -2000,
    };

    let attack_index = /* search_for_attack(&envelope_deriv, &search_settings) */ 50;
    let release_index = search_for_release(&sample_norm.as_slice(), &envelope_deriv, &search_settings);

    let loop_search_settings = LoopSettings {
        derivative_threshold: 0.06,
        min_loop_length: 1.5,
        distance_between_loops: 0.2,
        quality_factor: 10.0,
        final_pass_count: 1000,
    };

    let loop_point = find_loop_point(
        &loop_search_settings,
        &sample,
        freq,
        sample_rate,
        attack_index,
        release_index,
    );

    let note = (12.0 * f64::log2(freq / 440.0) + 69.0).round() as u8;
    let note_freq = 440.0 * 2_f64.powf((note as i16 - 69) as f64 / 12.0);

    let cents = (1200.0 * f64::log2(freq / note_freq)).round() as i16;

    SampleMetadata {
        attack_index,
        release_index,
        loop_start: loop_point.0,
        loop_end: loop_point.1,
        note,
        cents,
    }
}

#[test]
fn envelopes_idx_test() {
    let foo = Sample::load_resource(&PathBuf::from(
        "/home/mason/python/dsp/sample-analysis/test-samples/069-A-nt.wav",
    ))
    .unwrap();

    let audio = foo.buffer.audio_raw;

    calc_sample_metadata(&audio, foo.buffer.sample_rate, None);

    // println!("{:?}", resample_to(&amp, 400));

    //envelopes_idx(&DVector::from(test_sig), 10);
}
