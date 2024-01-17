use common::resource_manager::{serialize_resource_content, ResourceId, ResourceIndex, ResourceManager};
use common::traits::TryRef;
use serde::Serialize;
use sound_engine::{sampling::rank::Rank, MonoSample};

#[derive(Default, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Resources {
    pub samples: ResourceManager<MonoSample>,
    pub ranks: ResourceManager<Rank>,
    #[serde(serialize_with = "serialize_resource_content")]
    pub ui: ResourceManager<String>,
}

#[derive(Debug)]
pub enum Resource<'a> {
    Sample(&'a MonoSample),
    Rank(&'a Rank),
    Ui(&'a String),
    NotFound,
}

impl<'a> TryRef<MonoSample> for Resource<'a> {
    type Error = ();

    fn try_ref(&self) -> Result<&MonoSample, Self::Error> {
        match self {
            Self::Sample(x) => Ok(&x),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ResourceType {
    Sample,
    Rank,
    Ui,
}

#[derive(Debug, Clone, Copy)]
pub struct ResourceTypeAndIndex(pub ResourceType, pub ResourceIndex);

impl Resources {
    pub fn get_resource_index(&self, resource_id: &ResourceId) -> Option<ResourceTypeAndIndex> {
        match resource_id.namespace.as_str() {
            "ranks" => self
                .ranks
                .get_index(&resource_id.resource)
                .and_then(|x| Some(ResourceTypeAndIndex(ResourceType::Rank, x))),
            "samples" => self
                .samples
                .get_index(&resource_id.resource)
                .and_then(|x| Some(ResourceTypeAndIndex(ResourceType::Sample, x))),
            "ui" => self
                .ui
                .get_index(&resource_id.resource)
                .and_then(|x| Some(ResourceTypeAndIndex(ResourceType::Ui, x))),
            _ => None,
        }
    }

    pub fn get_resource(&self, type_and_index: &ResourceTypeAndIndex) -> Option<Resource> {
        let ResourceTypeAndIndex(resource_type, resource_index) = &type_and_index;

        match resource_type {
            ResourceType::Sample => {
                if let Some(sample) = self.samples.borrow_resource(*resource_index) {
                    return Some(Resource::Sample(sample));
                }
            }
            ResourceType::Rank => {
                if let Some(rank) = self.ranks.borrow_resource(*resource_index) {
                    return Some(Resource::Rank(rank));
                }
            }
            ResourceType::Ui => {
                if let Some(ui) = self.ui.borrow_resource(*resource_index) {
                    return Some(Resource::Ui(ui));
                }
            }
        }

        None
    }

    pub fn reset(&mut self) {
        self.ranks.clear();
        self.samples.clear();
        self.ui.clear();
    }
}
