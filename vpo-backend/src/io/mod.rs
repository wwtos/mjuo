use std::{
    fs,
    path::{Path, PathBuf},
};

use asset_manager::{Asset, AssetManager};
use node_engine::{
    errors::{IOSnafu, JsonParserSnafu, NodeError},
    global_state::GlobalState,
    state::NodeEngineState,
};
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

fn load_assets<T: Asset>(path: &Path, assets: &mut AssetManager<T>) {
    let asset_list: Vec<(PathBuf, String)> = WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| {
            if let Ok(res) = e {
                if res.metadata().unwrap().is_file() {
                    Some(res)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .map(|asset| {
            let asset_key = asset.path().strip_prefix(path).unwrap().to_string_lossy().to_string();
            (PathBuf::from(asset.path()), asset_key)
        })
        .collect();

    for (asset_src, asset_key) in asset_list {
        println!("Loading: {:?}", asset_key);
        assets.request_asset(asset_key, &asset_src).unwrap();
    }
}

pub fn load(path: &Path, state: &mut NodeEngineState, global_state: &mut GlobalState) -> Result<(), NodeError> {
    *state = NodeEngineState::new(&global_state);
    global_state.assets.samples.clear();

    load_assets(&path.join("samples"), &mut global_state.assets.samples);

    let json_raw = fs::read_to_string(path.join("state.json")).context(IOSnafu)?;
    let mut json: Value = serde_json::from_str(&json_raw).context(JsonParserSnafu)?;

    // TODO: version handling and migrations here
    state.apply_json(json["state"].take(), global_state)?;

    Ok(())
}
