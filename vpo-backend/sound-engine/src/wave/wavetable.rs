use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Wavetable {
    #[serde(skip)]
    pub wavetable: Vec<f32>,
}
