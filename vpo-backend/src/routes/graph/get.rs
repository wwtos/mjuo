use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{errors::NodeError, global_state::GlobalState, state::NodeEngineState};
use serde_json::Value;

use crate::{
    util::{send_graph_updates, send_registry_updates},
    RouteReturn,
};

pub fn route(
    msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    _global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    let graph_index = msg["payload"]["graphIndex"]
        .as_u64()
        .ok_or(NodeError::PropertyMissingOrMalformed {
            property_name: "graphIndex".to_string(),
        })?;

    send_graph_updates(state, graph_index, to_server)?;
    send_registry_updates(state.get_registry(), to_server).unwrap();

    Ok(None)
}
