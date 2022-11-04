use std::path::PathBuf;

use resource_manager::ResourceManager;
use serde::{ser::SerializeSeq, Serialize};
use sound_engine::{sampling::audio_loader::Sample, SoundConfig};

#[derive(Default)]
pub struct Resources {
    pub samples: ResourceManager<Sample>,
}

impl Serialize for Resources {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.samples.len()))?;

        for resource in self.samples.as_keys() {
            seq.serialize_element(&resource)?;
        }

        seq.end()
    }
}

#[derive(Serialize)]
pub struct GlobalState {
    pub active_project: Option<PathBuf>,
    pub sound_config: SoundConfig,
    pub resources: Resources,
}

impl GlobalState {
    pub fn new(sound_config: SoundConfig) -> GlobalState {
        GlobalState {
            active_project: None,
            resources: Resources::default(),
            sound_config,
        }
    }
}
