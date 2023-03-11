use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Wavetable {
    #[serde(skip)]
    pub sample_rate: u32,
    #[serde(skip)]
    pub wavetable: Vec<f32>,
}

impl Default for Wavetable {
    fn default() -> Self {
        Self {
            sample_rate: 1,
            wavetable: Vec::new(),
        }
    }
}
