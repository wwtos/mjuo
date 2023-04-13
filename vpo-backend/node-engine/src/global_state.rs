use std::path::PathBuf;

use resource_manager::ResourceManager;
use serde::Serialize;
use sound_engine::{
    sampling::{rank::Rank, sample::Pipe},
    wave::wavetable::Wavetable,
    SoundConfig,
};

#[derive(Default, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Resources {
    pub pipes: ResourceManager<Pipe>,
    pub wavetables: ResourceManager<Wavetable>,
    pub ranks: ResourceManager<Rank>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
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

    pub fn reset(&mut self) {
        self.resources.ranks.clear();
        self.resources.pipes.clear();
        self.resources.wavetables.clear();
    }
}
