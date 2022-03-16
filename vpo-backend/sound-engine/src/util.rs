pub mod sample_manager;
pub mod wav_reader;

pub fn lerp(start: f32, end: f32, amount: f32) -> f32 {
    (end - start) * amount + start
}
