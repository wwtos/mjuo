use std::{fs::File, io::Read, path::Path};

use resource_manager::{
    deserialize_resource_id, serialize_resource_id, IOSnafu, LoadingError, ParserSnafu, Resource, ResourceId,
};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct RankEntry {
    #[serde(serialize_with = "serialize_resource_id")]
    #[serde(deserialize_with = "deserialize_resource_id")]
    resource: ResourceId,
    note: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rank {
    samples: Vec<RankEntry>,
}

impl Resource for Rank {
    fn load_resource(path: &Path) -> Result<Rank, LoadingError>
    where
        Self: Sized,
    {
        let mut file = File::open(path).context(IOSnafu)?;
        let mut data = String::new();
        file.read_to_string(&mut data).context(IOSnafu)?;

        serde_json::from_str(&data).context(ParserSnafu)
    }
}
