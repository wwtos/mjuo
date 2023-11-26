use ipc::ipc_message::IpcMessage;
use node_engine::{global_state::GlobalState, graph_manager::GraphIndex, state::GraphState};
use serde_json::json;
use snafu::ResultExt;

use crate::{
    errors::{EngineError, NodeSnafu},
    Sender,
};

pub fn send_graph_updates(
    state: &mut GraphState,
    graph_index: GraphIndex,
    to_server: &Sender<IpcMessage>,
) -> Result<(), EngineError> {
    let graph = state
        .get_graph_manager()
        .get_graph_mut(graph_index)
        .context(NodeSnafu)?;
    let json = serde_json::to_value(&*graph).unwrap();

    // we don't care if there's no receivers to state updates
    let _ = to_server.send(IpcMessage::Json(json! {{
        "action": "graph/updateGraph",
        "payload": {
            "nodes": json["nodes"],
            "graphIndex": graph_index
        }
    }}));

    Ok(())
}

pub fn send_global_state_updates(
    global_state: &mut GlobalState,
    to_server: &Sender<IpcMessage>,
) -> Result<(), EngineError> {
    let json = global_state.to_json();

    let _ = to_server.send(IpcMessage::Json(json! {{
        "action": "state/updateGlobalState",
        "payload": json
    }}));

    Ok(())
}
