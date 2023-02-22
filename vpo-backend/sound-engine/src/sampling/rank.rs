use std::{fs::File, io::Read, path::Path};

use resource_manager::{
    deserialize_resource_id, serialize_resource_id, IOSnafu, LoadingError, ParserSnafu, Resource, ResourceId,
    TomlParserDeSnafu,
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

impl Resource for Rank {
    fn load_resource(path: &Path) -> Result<Rank, LoadingError>
    where
        Self: Sized,
    {
        let mut file = File::open(path).context(IOSnafu)?;
        let mut data = String::new();
        file.read_to_string(&mut data).context(IOSnafu)?;

        toml::from_str(&data).context(TomlParserDeSnafu)
    }
}
