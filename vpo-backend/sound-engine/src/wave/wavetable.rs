use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Wavetable {
    #[serde(skip)]
    pub wavetable: Vec<f32>,
}

impl Default for Wavetable {
    fn default() -> Self {
        Self { wavetable: Vec::new() }
    }
}
