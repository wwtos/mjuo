use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};

use node_engine::errors::LoadingSnafu;
use node_engine::{
    errors::{IOSnafu, JsonParserSnafu, NodeError},
    global_state::GlobalState,
    state::NodeEngineState,
};
use resource_manager::{LoadingError, Resource, ResourceManager};
use serde_json::{json, Value};
use snafu::ResultExt;
use walkdir::WalkDir;

pub fn save(state: &NodeEngineState, path: &Path) -> Result<(), NodeError> {
    let state = json!({
        "version": "0.3",
        "state": state.to_json()?
    });

    fs::write(
        path.join("state.json"),
        serde_json::to_string_pretty(&state).context(JsonParserSnafu)?,
    )
    .context(IOSnafu)?;

    Ok(())
}

fn load_assets<T>(path: &Path, assets: &mut ResourceManager<T>) -> Result<(), LoadingError>
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
                        if let Some("mp3" | "ogg" | "wav" | "flac") = extension.to_str() {
                            return Some(res);
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

    assets.request_resources_parallel(asset_list)
}

pub fn load(path: &Path, state: &mut NodeEngineState, global_state: &mut GlobalState) -> Result<(), NodeError> {
    *state = NodeEngineState::new(global_state);
    global_state.resources.samples.clear();

    load_assets(&path.join("samples"), &mut global_state.resources.samples).context(LoadingSnafu)?;

    let json_raw = fs::read_to_string(path.join("state.json")).context(IOSnafu)?;
    let mut json: Value = serde_json::from_str(&json_raw).context(JsonParserSnafu)?;

    // TODO: version handling and migrations here
    state.apply_json(json["state"].take(), global_state)?;

    Ok(())
}
