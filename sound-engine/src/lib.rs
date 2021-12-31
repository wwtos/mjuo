pub mod error;
pub mod node;
pub mod openal;

pub struct SoundConfig {
    sample_rate: u32,
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
