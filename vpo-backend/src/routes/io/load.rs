use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use futures::executor::block_on;
use ipc::ipc_message::IpcMessage;
use node_engine::{global_state::GlobalState, state::NodeState};
use serde_json::{json, Value};

use crate::{errors::EngineError, io::load, routes::RouteReturn, Sender};

pub fn route(
    msg: Value,
    to_server: &Sender<IpcMessage>,
    state: &mut NodeState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    if let Value::String(path) = &msg["payload"]["path"] {
        state.clear_history();
        load(Path::new(path), state, global_state)?;

        global_state.active_project = Some(PathBuf::from_str(path).unwrap());

        block_on(async {
            to_server
                .send(IpcMessage::Json(json! {{
                    "action": "io/loaded",
                }}))
                .await
        })
        .unwrap();

        Ok(None)
    } else {
        Err(EngineError::PropertyMissingOrMalformed {
            property_name: "payload.path".to_string(),
        })
    }
}
