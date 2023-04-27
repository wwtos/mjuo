use ipc::ipc_message::IPCMessage;
use node_engine::{global_state::GlobalState, state::NodeState};
use serde_json::Value;
use snafu::ResultExt;

use crate::{
    errors::{EngineError, NodeSnafu},
    routes::RouteReturn,
    util::send_graph_updates,
    Sender,
};

pub fn route(
    _msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    println!("undo");
    let (graphs_changed, ..) = state.undo(global_state).context(NodeSnafu)?;

    for graph_index in graphs_changed {
        send_graph_updates(state, graph_index, to_server)?;
    }

    Ok(None)
}
