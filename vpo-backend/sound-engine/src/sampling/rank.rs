use std::collections::BTreeMap;

use resource_manager::{deserialize_resource_id, serialize_resource_id, ResourceId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Pipe {
    #[serde(serialize_with = "serialize_resource_id")]
    #[serde(deserialize_with = "deserialize_resource_id")]
    pub resource: ResourceId,
    pub note: u8,
    pub cents: i16,
    pub amplitude: f32,
    pub loop_start: usize,
    pub loop_end: usize,
    pub decay_index: usize,
    pub release_index: usize,
    pub crossfade: usize,
}

#[derive(Default, Debug)]
pub struct Rank {
    pub pipes: BTreeMap<u8, Pipe>,
    pub name: String,
}
