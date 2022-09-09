use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use async_std::{channel::Sender, task::block_on};
use ipc::ipc_message::IPCMessage;
use node_engine::{errors::NodeError, state::NodeEngineState};
use serde_json::{json, Value};

use crate::{io::save, state::GlobalState, RouteReturn};

pub fn route(
    msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    if let Some(project_path) = &global_state.active_project {
        save(state, project_path)?;
    } else if let Value::String(path) = &msg["payload"]["path"] {
        save(state, Path::new(path))?;

        global_state.active_project = Some(PathBuf::from_str(path).unwrap());
    } else {
        block_on(async {
            to_server
                .send(IPCMessage::Json(json! {{
                    "action": "io/getSaveLocation",
                }}))
                .await
        })
        .unwrap();
    }

    Ok(None)
}