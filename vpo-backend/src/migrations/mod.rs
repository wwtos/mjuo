use std::{fs, path::PathBuf};

use lazy_static::lazy_static;
use semver::Version;
use serde_json::Value;
use snafu::ResultExt;

use crate::errors::{EngineError, IoSnafu, JsonParserSnafu};

type MigrationFn = Box<dyn Fn(PathBuf) -> Result<(), EngineError> + Send + Sync + 'static>;

pub struct Migration {
    pub version_from: Version,
    pub version_to: Version,
    pub migrate: MigrationFn,
}

lazy_static! {
    pub static ref MIGRATIONS: [Migration; 0] = {
        [
            // Migration {
            //     version_from: Version::parse("0.3.0").unwrap(),
            //     version_to: Version::parse("0.4.0").unwrap(),
            //     migrate: Box::new(m_0000_add_polyphonic_prop::migrate),
            // }
        ]
    };
}

pub fn migrate(project: PathBuf) -> Result<(), EngineError> {
    // get version
    let json_raw = fs::read_to_string(project.join("state.json")).context(IoSnafu)?;
    let json: Value = serde_json::from_str(&json_raw).context(JsonParserSnafu)?;

    let version_str = json["version"]
        .as_str()
        .ok_or(EngineError::PropertyMissingOrMalformed {
            property_name: "version".into(),
        })?;

    let version = Version::parse(version_str).map_err(|_| EngineError::PropertyMissingOrMalformed {
        property_name: "version".into(),
    })?;

    // find this version in the list
    let version_index = MIGRATIONS
        .iter()
        .position(|migration| migration.version_from == version)
        .ok_or(EngineError::VersionError { version })?;

    let migrations_to_apply = &MIGRATIONS[version_index..MIGRATIONS.len()];

    for migration in migrations_to_apply {
        (migration.migrate)(project.clone())?;
    }

    Ok(())
}
