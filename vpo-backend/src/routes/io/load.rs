use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use futures::executor::block_on;
use ipc::ipc_message::IPCMessage;
use node_engine::{errors::NodeError, global_state::GlobalState, state::NodeEngineState};
use serde_json::{json, Value};

use crate::{io::load, routes::RouteReturn, Sender};

pub fn route(
    msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    if let Value::String(path) = &msg["payload"]["path"] {
        state.clear_history();
        load(Path::new(path), state, global_state)?;

        global_state.active_project = Some(PathBuf::from_str(path).unwrap());

        block_on(async {
            to_server
                .send(IPCMessage::Json(json! {{
                    "action": "io/loaded",
                }}))
                .await
        })
        .unwrap();

        Ok(None)
    } else {
        Err(NodeError::PropertyMissingOrMalformed {
            property_name: "payload.path".to_string(),
        })
    }
}
