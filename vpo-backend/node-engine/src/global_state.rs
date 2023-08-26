use std::any::Any;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use resource_manager::{serialize_resource_content, ResourceId, ResourceIndex, ResourceManager};
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

#[derive(Debug, Clone)]
pub enum ResourceType {
    Sample,
    Rank,
    Ui,
}

impl Resources {
    pub fn get_resource_index(&self, resource_id: &ResourceId) -> Option<(ResourceType, ResourceIndex)> {
        match resource_id.namespace.as_str() {
            "ranks" => self
                .ranks
                .get_index(&resource_id.resource)
                .and_then(|x| Some((ResourceType::Rank, x))),
            "samples" => self
                .samples
                .get_index(&resource_id.resource)
                .and_then(|x| Some((ResourceType::Sample, x))),
            "ui" => self
                .ui
                .get_index(&resource_id.resource)
                .and_then(|x| Some((ResourceType::Ui, x))),
            _ => None,
        }
    }

    pub fn get_any(&self, resource_type: &ResourceType, resource_index: ResourceIndex) -> Option<&dyn Any> {
        match resource_type {
            ResourceType::Sample => {
                if let Some(sample) = self.samples.borrow_resource(resource_index) {
                    return Some(sample as &dyn Any);
                }
            }
            ResourceType::Rank => {
                if let Some(rank) = self.ranks.borrow_resource(resource_index) {
                    return Some(rank as &dyn Any);
                }
            }
            ResourceType::Ui => {
                if let Some(ui) = self.ui.borrow_resource(resource_index) {
                    return Some(ui as &dyn Any);
                }
            }
        }

        None
    }
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
