use std::{fs, path::Path};

use node_engine::{errors::NodeError, state::NodeEngineState};
use serde_json::{json, Value};

pub fn save(state: &NodeEngineState, path: &Path) -> Result<(), NodeError> {
    let state = json!({
        "version": "0.3",
        "state": state.to_json()?
    });

    fs::write(path.join("state.json"), serde_json::to_string_pretty(&state)?)?;

    Ok(())
}

pub fn load(state: &mut NodeEngineState, path: &Path) -> Result<(), NodeError> {
    let json_raw = fs::read_to_string(path.join("state.json"))?;
    let mut json: Value = serde_json::from_str(&json_raw)?;

    // TODO: version handling and migrations here

    state.apply_json(json["state"].take())?;

    Ok(())
}
