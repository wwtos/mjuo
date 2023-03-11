use std::{fs::File, io::Read, path::Path};

use resource_manager::{
    deserialize_resource_id, serialize_resource_id, IOSnafu, LoadingError, Resource, ResourceId, TomlParserDeSnafu,
};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct RankEntry {
    #[serde(serialize_with = "serialize_resource_id")]
    #[serde(deserialize_with = "deserialize_resource_id")]
    pub resource: ResourceId,
    pub note: u8,
}

impl Default for RankEntry {
    fn default() -> Self {
        RankEntry {
            resource: ResourceId {
                namespace: "ranks".into(),
                resource: "none".into(),
            },
            note: 0,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Rank {
    pub samples: Vec<RankEntry>,
}
