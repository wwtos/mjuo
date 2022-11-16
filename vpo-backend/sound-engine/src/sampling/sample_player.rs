pub enum SampleMessage {
    Exit,
}

pub struct SamplePlayer {}

impl SamplePlayer {
    pub fn new() -> SamplePlayer {
        SamplePlayer {}
    }
}

impl Default for SamplePlayer {
    fn default() -> Self {
        Self::new()
    }
}
