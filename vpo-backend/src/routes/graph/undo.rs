use ipc::ipc_message::IPCMessage;
use node_engine::{errors::NodeError, global_state::GlobalState, state::NodeEngineState};
use serde_json::Value;

use crate::{routes::RouteReturn, util::send_graph_updates, Sender};

pub fn route(
    _msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    println!("undo");
    let (graphs_changed, _) = state.undo(global_state)?;

    for graph_index in graphs_changed {
        send_graph_updates(state, graph_index, to_server)?;
    }

    Ok(None)
}
