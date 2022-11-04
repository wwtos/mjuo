use std::path::PathBuf;

use resource_manager::ResourceManager;
use sound_engine::{sampling::audio_loader::Sample, SoundConfig};

#[derive(Default)]
pub struct Assets {
    pub samples: ResourceManager<Sample>,
}

pub struct GlobalState {
    pub active_project: Option<PathBuf>,
    pub sound_config: SoundConfig,
    pub assets: Assets,
}

impl GlobalState {
    pub fn new(sound_config: SoundConfig) -> GlobalState {
        GlobalState {
            active_project: None,
            assets: Assets::default(),
            sound_config,
        }
    }
}
