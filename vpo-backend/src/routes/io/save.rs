use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use ipc::ipc_message::IpcMessage;
use node_engine::{global_state::GlobalState, state::NodeState};
use serde_json::{json, Value};
use tokio::sync::broadcast;

use crate::{errors::EngineError, io::save, routes::RouteReturn};

pub async fn route(
    msg: Value,
    to_server: &broadcast::Sender<IpcMessage>,
    state: &mut NodeState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    if let Some(project_path) = &global_state.active_project {
        save(state, project_path)?;
    } else if let Value::String(path) = &msg["payload"]["path"] {
        save(state, Path::new(path))?;

        global_state.active_project = Some(PathBuf::from_str(path).unwrap());
    } else {
        to_server
            .send(IpcMessage::Json(json! {{
                "action": "io/getSaveLocation",
            }}))
            .unwrap();
    }

    Ok(None)
}
