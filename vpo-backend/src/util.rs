use futures::executor::block_on;
use ipc::ipc_message::IpcMessage;
use node_engine::{
    global_state::GlobalState, graph_manager::GraphIndex, socket_registry::SocketRegistry, state::NodeState,
};
use serde_json::json;
use snafu::ResultExt;

use crate::{
    errors::{EngineError, JsonParserSnafu, NodeSnafu},
    Sender,
};

pub fn send_graph_updates(
    state: &mut NodeState,
    graph_index: GraphIndex,
    to_server: &Sender<IpcMessage>,
) -> Result<(), EngineError> {
    let graph = state
        .get_graph_manager()
        .get_graph(graph_index)
        .context(NodeSnafu)?
        .graph
        .borrow_mut();
    let json = serde_json::to_value(&*graph).unwrap();

    block_on(async {
        to_server
            .send(IpcMessage::Json(json! {{
                "action": "graph/updateGraph",
                "payload": {
                    "nodes": json["nodes"],
                    "graphIndex": graph_index
                }
            }}))
            .await
    })
    .unwrap();

    Ok(())
}

pub fn send_registry_updates(registry: &SocketRegistry, to_server: &Sender<IpcMessage>) -> Result<(), EngineError> {
    let json = serde_json::to_value(registry).context(JsonParserSnafu)?;

    block_on(async {
        to_server
            .send(IpcMessage::Json(json! {{
                "action": "registry/updateRegistry",
                "payload": json
            }}))
            .await
    })
    .unwrap();

    Ok(())
}

pub fn send_global_state_updates(
    global_state: &mut GlobalState,
    to_server: &Sender<IpcMessage>,
) -> Result<(), EngineError> {
    let json = global_state.to_json();

    block_on(async {
        to_server
            .send(IpcMessage::Json(json! {{
                "action": "state/updateGlobalState",
                "payload": json
            }}))
            .await
    })
    .unwrap();

    Ok(())
}
