use std::{fs, path::Path};

use node_engine::{errors::NodeError, state::NodeEngineState};
use serde_json::json;

pub fn save(state: &NodeEngineState, path: &Path) -> Result<(), NodeError> {
    let state = json!({
        "version": "0.2",
        "state": state.to_json()?
    });

    fs::write(path.join("state.json"), serde_json::to_string_pretty(&state)?)?;

    Ok(())
}
