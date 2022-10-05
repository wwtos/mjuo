pub mod backend;
pub mod error;
pub mod midi;
pub mod node;
pub mod openal;
pub mod ringbuffer;
pub mod sampling;
pub mod util;
pub mod wave;

pub type SamplePoint = i16;

#[derive(Clone)]
pub struct SoundConfig {
    pub sample_rate: u32,
}

pub struct MonoSample {
    pub audio_raw: Vec<f32>,
    pub sample_rate: u32,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

pub mod constants {
    #[allow(clippy::excessive_precision)]
    pub const PI: f32 = 3.14159265358979323846264338327950288f32;
    pub const TWO_PI: f32 = PI * 2.0;
    pub const BUFFER_SIZE: usize = 256;
    pub const SAMPLE_RATE: u32 = 48_000;
}
