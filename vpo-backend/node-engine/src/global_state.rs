use std::path::PathBuf;

use asset_manager::AssetManager;
use sound_engine::MonoSample;

pub struct GlobalState {
    pub active_project: Option<PathBuf>,
    pub samples: AssetManager<MonoSample>,
}

impl GlobalState {
    pub fn new() -> GlobalState {
        GlobalState {
            active_project: None,
            samples: AssetManager::new(),
        }
    }
}
