pub mod sample_manager;
pub mod wav_reader;

pub fn lerp(start: f32, end: f32, amount: f32) -> f32 {
    (end - start) * amount + start
}

pub fn db_to_amplitude(db: f32) -> f32 {
    10_f32.powf(db / 20.0)
}

pub fn amplitude_to_db(amp: f32) -> f32 {
    20.0 * f32::log10(amp)
}
