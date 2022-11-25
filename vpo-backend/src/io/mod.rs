use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;

use node_engine::errors::LoadingSnafu;
use node_engine::{
    errors::{IOSnafu, JsonParserSnafu, NodeError},
    global_state::GlobalState,
    state::NodeEngineState,
};
use resource_manager::{LoadingError, Resource, ResourceManager};
use semver::Version;
use serde_json::{json, Value};
use snafu::ResultExt;
use walkdir::WalkDir;

use crate::migrations::migrate;

const AUDIO_EXTENSIONS: &'static [&'static str] = &["ogg", "wav", "mp3", "flac"];
lazy_static! {
    pub static ref VERSION: Version = Version::parse("0.4.0").unwrap();
}

pub fn save(state: &NodeEngineState, path: &Path) -> Result<(), NodeError> {
    let state = json!({
        "version": VERSION.to_string(),
        "state": state.to_json()?
    });

    fs::write(
        path.join("state.json"),
        serde_json::to_string_pretty(&state).context(JsonParserSnafu)?,
    )
    .context(IOSnafu)?;

    Ok(())
}

fn load_resources<T>(
    path: &Path,
    resources: &mut ResourceManager<T>,
    valid_extensions: &[&str],
) -> Result<(), LoadingError>
where
    T: Resource + Send + Sync + Debug + 'static,
{
    let asset_list = WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| {
            if let Ok(res) = e {
                if res.metadata().unwrap().is_file() {
                    if let Some(extension) = res.path().extension() {
                        if let Some(extension) = extension.to_str() {
                            if valid_extensions.contains(&extension) {
                                return Some(res);
                            }
                        }
                    }
                }
            }

            None
        })
        .map(|asset| {
            let asset_key = asset.path().strip_prefix(path).unwrap().to_string_lossy().to_string();
            (asset_key, PathBuf::from(asset.path()))
        });

    resources.watch_resources(asset_list)
}

pub fn load(path: &Path, state: &mut NodeEngineState, global_state: &mut GlobalState) -> Result<(), NodeError> {
    let json_raw = fs::read_to_string(path.join("state.json")).context(IOSnafu)?;
    let json: Value = serde_json::from_str(&json_raw).context(JsonParserSnafu)?;

    if let Some(version) = json["version"].as_str() {
        if version != VERSION.to_string() {
            migrate(PathBuf::from(path))?;
        }
    }

    let json_raw = fs::read_to_string(path.join("state.json")).context(IOSnafu)?;
    let mut json: Value = serde_json::from_str(&json_raw).context(JsonParserSnafu)?;

    *state = NodeEngineState::new(global_state);
    global_state.resources.samples.clear();

    load_resources(
        &path.join("samples"),
        &mut global_state.resources.samples,
        AUDIO_EXTENSIONS,
    )
    .context(LoadingSnafu)?;
    load_resources(
        &path.join("wavetables"),
        &mut global_state.resources.wavetables,
        AUDIO_EXTENSIONS,
    )
    .context(LoadingSnafu)?;

    // TODO: version handling and migrations here
    state.apply_json(json["state"].take(), global_state)?;

    Ok(())
}
