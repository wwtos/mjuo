use std::{fs, path::PathBuf};

use lazy_static::lazy_static;
use node_engine::errors::{IOSnafu, JsonParserSnafu, NodeError};
use semver::Version;
use serde_json::Value;
use snafu::ResultExt;

pub mod m_0000_add_polyphonic_prop;

type MigrationFn = Box<dyn Fn(PathBuf) -> Result<(), NodeError> + Send + Sync + 'static>;

pub struct Migration {
    pub version_from: Version,
    pub version_to: Version,
    pub migrate: MigrationFn,
}

lazy_static! {
    pub static ref MIGRATIONS: [Migration; 1] = {
        [Migration {
            version_from: Version::parse("0.3.0").unwrap(),
            version_to: Version::parse("0.4.0").unwrap(),
            migrate: Box::new(m_0000_add_polyphonic_prop::migrate),
        }]
    };
}

pub fn migrate(project: PathBuf) -> Result<(), NodeError> {
    // get version
    let json_raw = fs::read_to_string(project.join("state.json")).context(IOSnafu)?;
    let json: Value = serde_json::from_str(&json_raw).context(JsonParserSnafu)?;

    let version_str = json["version"].as_str().ok_or(NodeError::PropertyMissingOrMalformed {
        property_name: "version".into(),
    })?;

    let version = Version::parse(&version_str).map_err(|_| NodeError::PropertyMissingOrMalformed {
        property_name: "version".into(),
    })?;

    // find this version in the list
    let version_index = MIGRATIONS
        .iter()
        .position(|migration| migration.version_from == version)
        .ok_or(NodeError::VersionError { version: version })?;

    let migrations_to_apply = &MIGRATIONS[version_index..MIGRATIONS.len()];

    for migration in migrations_to_apply {
        (migration.migrate)(project.clone())?;
    }

    Ok(())
}
