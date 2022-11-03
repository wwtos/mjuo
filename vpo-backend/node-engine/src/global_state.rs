use std::path::PathBuf;

use asset_manager::AssetManager;
use sound_engine::{MonoSample, SoundConfig};

#[derive(Default)]
pub struct Assets {
    pub samples: AssetManager<MonoSample>,
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
