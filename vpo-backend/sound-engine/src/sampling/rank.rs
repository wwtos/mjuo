use resource_manager::{deserialize_resource_id, serialize_resource_id, ResourceId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RankEntry {
    #[serde(serialize_with = "serialize_resource_id")]
    #[serde(deserialize_with = "deserialize_resource_id")]
    resource_id: ResourceId,
    note: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rank {
    entries: Vec<RankEntry>,
}

impl Rank {}
