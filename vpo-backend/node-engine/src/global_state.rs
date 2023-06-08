use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use resource_manager::{serialize_resource_content, ResourceManager};
use serde::Serialize;
use serde_json::{json, Value};
use sound_engine::{sampling::rank::Rank, MonoSample, SoundConfig};

#[derive(Default, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Resources {
    pub samples: ResourceManager<MonoSample>,
    pub ranks: ResourceManager<Rank>,
    #[serde(serialize_with = "serialize_resource_content")]
    pub ui: ResourceManager<String>,
}

#[derive(Debug)]
pub struct GlobalState {
    pub active_project: Option<PathBuf>,
    pub import_folder: Option<PathBuf>,
    pub sound_config: SoundConfig,
    pub resources: Arc<RwLock<Resources>>,
}

impl GlobalState {
    pub fn new(sound_config: SoundConfig) -> GlobalState {
        GlobalState {
            active_project: None,
            import_folder: None,
            resources: Arc::new(RwLock::new(Resources::default())),
            sound_config,
        }
    }

    pub fn reset(&mut self) {
        let mut resources = self.resources.write().unwrap();

        resources.ranks.clear();
        resources.samples.clear();
        resources.ui.clear();
    }

    pub fn to_json(&self) -> Value {
        let resources = self.resources.read().unwrap();

        json!({
            "activeProject": self.active_project,
            "soundConfig": self.sound_config,
            "resources": *resources
        })
    }

    pub fn project_directory(&self) -> Option<PathBuf> {
        self.active_project
            .as_ref()
            .and_then(|project| project.parent())
            .map(|dir| dir.into())
    }
}
