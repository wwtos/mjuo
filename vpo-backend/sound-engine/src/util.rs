pub mod interpolate;
pub mod wav_reader;

pub fn db_to_gain(db: f32) -> f32 {
    10_f32.powf(db / 20.0)
}

pub fn gain_to_db(amp: f32) -> f32 {
    20.0 * f32::log10(amp)
}

pub fn cents_to_detune(cents: f32) -> f32 {
    2_f32.powf(cents / 1200.0)
}

pub fn detune_to_cents(detune: f32) -> f32 {
    f32::log2(detune) * 1200.0
}
