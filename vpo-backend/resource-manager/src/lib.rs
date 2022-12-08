use std::{
    collections::HashMap,
    fmt::Debug,
    io,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread::available_parallelism,
};

use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};
use snafu::Snafu;
use threadpool::ThreadPool;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum LoadingError {
    #[snafu(display("IO error: {source}"))]
    IOError { source: io::Error },
    #[snafu(display("Parser error: {source}"))]
    ParserError { source: serde_json::Error },
    #[snafu(display("TOML serialization error: {source}"))]
    TomlParserSerError { source: toml::ser::Error },
    #[snafu(display("TOML deserialization error: {source}"))]
    TomlParserDeError { source: toml::de::Error },
    #[snafu(display("Unknown error: {source}"))]
    Other { source: Box<dyn std::error::Error> },
}

pub trait Resource {
    fn load_resource(path: &Path) -> Result<Self, LoadingError>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceIndex {
    pub index: usize,
    pub generation: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceId {
    pub namespace: String,
    pub resource: String,
}

impl ResourceId {
    pub fn from_str(id: &str) -> Option<ResourceId> {
        let foo: Vec<&str> = id.split(":").take(2).collect();

        if foo.len() != 2 {
            None
        } else {
            Some(ResourceId {
                namespace: foo[0].to_string(),
                resource: foo[1].to_string(),
            })
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.namespace, self.resource)
    }
}

pub fn serialize_resource_id<S>(resource_id: &ResourceId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&resource_id.to_string())
}

pub fn deserialize_resource_id<'de, D>(deserializer: D) -> Result<ResourceId, D::Error>
where
    D: Deserializer<'de>,
{
    let resource_id: String = serde::Deserialize::deserialize(deserializer)?;

    Ok(ResourceId::from_str(&resource_id).unwrap())
}

pub enum PossibleResource<A: Resource> {
    Some(A, u32),
    None(u32),
}

#[derive(Default)]
pub struct ResourceManager<A: Resource> {
    resources: Vec<PossibleResource<A>>,
    resource_mapping: HashMap<String, ResourceIndex>,
    resources_to_watch: Vec<(String, PathBuf)>,
}

impl<A> Serialize for ResourceManager<A>
where
    A: Resource + Debug + Send + Sync + 'static,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;

        for resource in self.as_keys() {
            seq.serialize_element(&resource)?;
        }

        seq.end()
    }
}

impl<A> ResourceManager<A>
where
    A: Resource + Debug + Send + Sync + 'static,
{
    pub fn new() -> ResourceManager<A> {
        ResourceManager {
            resources: Vec::new(),
            resource_mapping: HashMap::new(),
            resources_to_watch: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.resource_mapping.len()
    }

    pub fn is_empty(&self) -> bool {
        self.resource_mapping.is_empty()
    }

    pub fn as_keys(&self) -> Vec<String> {
        self.resource_mapping.keys().cloned().collect()
    }

    fn add_resource(&mut self, resource: A) -> ResourceIndex {
        // check if there's an opening
        let possible_opening = self
            .resources
            .iter()
            .position(|resource| !matches!(resource, PossibleResource::Some(..)));

        // put the new resource in the opening
        if let Some(opening) = possible_opening {
            let new_generation = match self.resources[opening] {
                PossibleResource::Some(..) => unreachable!(),
                PossibleResource::None(generation) => generation + 1,
            };

            self.resources[opening] = PossibleResource::Some(resource, new_generation);

            ResourceIndex {
                index: opening,
                generation: new_generation,
            }
        } else {
            // else, expand the resource length
            let index = self.resources.len();
            let new_generation = 0;

            self.resources.push(PossibleResource::Some(resource, new_generation));

            ResourceIndex {
                index,
                generation: new_generation,
            }
        }
    }

    pub fn get_index(&self, key: &str) -> Option<ResourceIndex> {
        self.resource_mapping.get(key).copied()
    }

    fn request_resources_parallel<I>(&mut self, resources_to_load: I) -> Result<(), LoadingError>
    where
        I: Iterator<Item = (String, PathBuf)>,
    {
        // create the structures to populate
        let resources: Arc<Mutex<HashMap<String, A>>> = Arc::new(Mutex::new(HashMap::new()));

        let pool = ThreadPool::new(available_parallelism().unwrap_or(NonZeroUsize::new(4).unwrap()).into());

        let existing_resources: Arc<Mutex<Vec<String>>> =
            Arc::new(Mutex::new(self.resource_mapping.keys().cloned().collect()));

        for resource_to_load in resources_to_load {
            let resources_cloned = Arc::clone(&resources);
            let existing_resources_cloned = Arc::clone(&existing_resources);
            let (key, location) = resource_to_load.clone();

            pool.execute(move || {
                // check if we've already loaded this resource
                let existing_resources = existing_resources_cloned.lock().unwrap();

                if existing_resources.iter().any(|x| x == &key) {
                    return;
                }

                // else, load and register it
                let new_resource = A::load_resource(&location).unwrap();
                println!("Loaded: {}", location.to_string_lossy());

                resources_cloned.lock().unwrap().insert(key, new_resource);
            });
        }

        pool.join();

        let new_resources = Arc::try_unwrap(resources).unwrap().into_inner().unwrap();

        for (key, resource) in new_resources.into_iter() {
            let resource_index = self.add_resource(resource);
            self.resource_mapping.insert(key, resource_index);
        }

        Ok(())
    }

    pub fn watch_resources<I>(&mut self, resources_to_load: I) -> Result<(), LoadingError>
    where
        I: Iterator<Item = (String, PathBuf)>,
    {
        let mut resources_to_watch = Vec::new();

        self.request_resources_parallel(resources_to_load.map(|resource| {
            resources_to_watch.push(resource.clone());

            resource
        }))?;

        self.resources_to_watch.extend(resources_to_watch);

        Ok(())
    }

    pub fn borrow_resource(&self, index: ResourceIndex) -> Option<&A> {
        if index.index >= self.resources.len() {
            None
        } else {
            match &self.resources[index.index] {
                PossibleResource::Some(resource, generation) => {
                    if index.generation == *generation {
                        Some(resource)
                    } else {
                        None
                    }
                }
                PossibleResource::None(_) => None,
            }
        }
    }

    pub fn clear(&mut self) {
        self.resource_mapping.clear();
        self.resources.clear();
    }
}
