use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{errors::NodeError, global_state::GlobalState, state::NodeEngineState};
use serde_json::Value;

use crate::{util::send_graph_updates, RouteReturn};

pub fn route(
    _msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    println!("undo");
    let graphs_changed = state.undo(global_state)?;

    for graph_index in graphs_changed {
        send_graph_updates(state, graph_index, to_server)?;
    }

    Ok(None)
}
