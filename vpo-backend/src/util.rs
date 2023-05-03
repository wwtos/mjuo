use ipc::ipc_message::IpcMessage;
use node_engine::{
    global_state::GlobalState, graph_manager::GraphIndex, socket_registry::SocketRegistry, state::NodeState,
};
use serde_json::json;
use snafu::ResultExt;
use tokio::sync::broadcast;

use crate::errors::{EngineError, JsonParserSnafu, NodeSnafu};

pub fn send_graph_updates(
    state: &mut NodeState,
    graph_index: GraphIndex,
    to_server: &broadcast::Sender<IpcMessage>,
) -> Result<(), EngineError> {
    let graph = state
        .get_graph_manager()
        .get_graph(graph_index)
        .context(NodeSnafu)?
        .graph
        .borrow_mut();
    let json = serde_json::to_value(&*graph).unwrap();

    to_server
        .send(IpcMessage::Json(json! {{
            "action": "graph/updateGraph",
            "payload": {
                "nodes": json["nodes"],
                "graphIndex": graph_index
            }
        }}))
        .unwrap();

    Ok(())
}

pub fn send_registry_updates(
    registry: &SocketRegistry,
    to_server: &broadcast::Sender<IpcMessage>,
) -> Result<(), EngineError> {
    let json = serde_json::to_value(registry).context(JsonParserSnafu)?;

    to_server
        .send(IpcMessage::Json(json! {{
            "action": "registry/updateRegistry",
            "payload": json
        }}))
        .unwrap();

    Ok(())
}

pub fn send_global_state_updates(
    global_state: &mut GlobalState,
    to_server: &broadcast::Sender<IpcMessage>,
) -> Result<(), EngineError> {
    let json = global_state.to_json();

    to_server
        .send(IpcMessage::Json(json! {{
            "action": "state/updateGlobalState",
            "payload": json
        }}))
        .unwrap();

    Ok(())
}
