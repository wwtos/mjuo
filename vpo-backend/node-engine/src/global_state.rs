use std::path::PathBuf;

use resource_manager::ResourceManager;
use serde::Serialize;
use sound_engine::{sampling::audio_loader::Sample, SoundConfig};

#[derive(Default, Serialize)]
pub struct Resources {
    pub samples: ResourceManager<Sample>,
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
