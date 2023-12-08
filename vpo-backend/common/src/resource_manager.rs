use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    str::FromStr,
};

use ddgg::{GenVec, Index};
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceIndex(Index);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceId {
    pub namespace: String,
    pub resource: String,
}

fn strip_empty_slashes(path: &str) -> String {
    let split = path.split('/');
    let mut result: Vec<&str> = vec![];

    for part in split {
        if !part.is_empty() {
            result.push(part);
        }
    }

    result.join("/")
}

#[derive(Debug)]
pub struct ResourceIdParserFailure(pub String);

impl Display for ResourceIdParserFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResourceIdParserFailure({})", self.0)
    }
}

impl FromStr for ResourceId {
    type Err = ResourceIdParserFailure;

    fn from_str(id: &str) -> Result<ResourceId, Self::Err> {
        let resource: Vec<&str> = id.split(':').take(2).collect();

        if resource.len() == 1 {
            Ok(ResourceId {
                namespace: strip_empty_slashes(resource[0]),
                resource: "".into(),
            })
        } else if resource.len() == 2 {
            Ok(ResourceId {
                namespace: strip_empty_slashes(resource[0]),
                resource: strip_empty_slashes(resource[1]),
            })
        } else {
            Err(ResourceIdParserFailure(id.to_string()))
        }
    }
}

impl Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.resource)
    }
}

impl ResourceId {
    pub fn concat(&self, other: &str) -> ResourceId {
        let stripped = strip_empty_slashes(other);

        ResourceId {
            namespace: self.namespace.clone(),
            resource: format!("{}/{}", self.resource, stripped),
        }
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

#[derive(Debug)]
pub struct ResourceManager<A> {
    resources: GenVec<A>,
    resource_mapping: HashMap<String, ResourceIndex>,
}

impl<A> Default for ResourceManager<A> {
    fn default() -> Self {
        ResourceManager {
            resources: GenVec::new(),
            resource_mapping: HashMap::new(),
        }
    }
}

impl<A> Serialize for ResourceManager<A> {
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

pub fn serialize_resource_content<S, A: Serialize>(
    resource_manager: &ResourceManager<A>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(resource_manager.len()))?;

    for (path, resource_index) in &resource_manager.resource_mapping {
        let resource = resource_manager
            .resources
            .get(resource_index.0)
            .expect("mapping destination to exist");

        map.serialize_entry(path, resource)?;
    }

    map.end()
}

impl<A> ResourceManager<A> {
    pub fn new() -> ResourceManager<A> {
        ResourceManager {
            resources: GenVec::new(),
            resource_mapping: HashMap::new(),
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

    pub fn extend(&mut self, mut other: ResourceManager<A>) {
        for (key, other_index) in other.resource_mapping {
            if let Some(other) = other.resources.remove(other_index.0) {
                self.add_resource(key, other);
            }
        }
    }

    pub fn add_resource(&mut self, key: String, resource: A) -> ResourceIndex {
        // if it already exists, be sure to remove the old one
        if let Some(_) = self.resource_mapping.get(&key) {
            self.remove_resource(&key);
        }

        let index = ResourceIndex(self.resources.add(resource));

        self.resource_mapping.insert(key, index);

        index
    }

    pub fn remove_resource(&mut self, key: &str) -> Option<A> {
        self.resource_mapping
            .remove(key)
            .and_then(|index| self.resources.remove(index.0))
    }

    pub fn get_index(&self, key: &str) -> Option<ResourceIndex> {
        self.resource_mapping.get(key).copied()
    }

    pub fn borrow_resource(&self, index: ResourceIndex) -> Option<&A> {
        self.resources.get(index.0)
    }

    pub fn borrow_resource_by_id(&self, id: &str) -> Option<&A> {
        self.get_index(id).and_then(|x| self.borrow_resource(x))
    }

    pub fn clear(&mut self) {
        self.resource_mapping.clear();
        self.resources.clear();
    }
}
