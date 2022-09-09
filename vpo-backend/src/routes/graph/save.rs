use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{errors::NodeError, state::StateManager};
use serde_json::Value;

use crate::{
    util::{send_graph_updates, send_registry_updates},
    RouteReturn,
};

pub fn route(
    msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut StateManager,
    global_state: &mut GlobalStsate,
) -> Result<Option<RouteReturn>, NodeError> {
    Ok(None)
}
