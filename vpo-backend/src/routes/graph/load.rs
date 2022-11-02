use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{errors::NodeError, global_state::GlobalState, state::NodeEngineState};
use serde_json::Value;

use crate::{io::load, RouteReturn};

pub fn route(
    msg: Value,
    _to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    if let Value::String(path) = &msg["payload"]["path"] {
        load(state, Path::new(path), global_state)?;

        global_state.active_project = Some(PathBuf::from_str(path).unwrap());

        Ok(None)
    } else {
        Err(NodeError::PropertyMissingOrMalformed {
            property_name: "payload.path".to_string(),
        })
    }
}
