use futures::executor::block_on;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::{JsonParserSnafu, NodeError},
    global_state::GlobalState,
    graph_manager::GraphIndex,
    socket_registry::SocketRegistry,
    state::NodeEngineState,
};
use serde_json::json;
use snafu::ResultExt;

use crate::Sender;

pub fn send_graph_updates(
    state: &mut NodeEngineState,
    graph_index: GraphIndex,
    to_server: &Sender<IPCMessage>,
) -> Result<(), NodeError> {
    let graph = state.get_graph_manager().get_graph(graph_index)?.graph.borrow_mut();
    let json = serde_json::to_value(&*graph).unwrap();

    block_on(async {
        to_server
            .send(IPCMessage::Json(json! {{
                "action": "graph/updateGraph",
                "payload": {
                    "nodes": json["nodes"],
                    "graphIndex": graph_index
                }
            }}))
            .await
    });

    Ok(())
}

pub fn send_registry_updates(registry: &SocketRegistry, to_server: &Sender<IPCMessage>) -> Result<(), NodeError> {
    let json = serde_json::to_value(registry).context(JsonParserSnafu)?;

    block_on(async {
        to_server
            .send(IPCMessage::Json(json! {{
                "action": "registry/updateRegistry",
                "payload": json
            }}))
            .await
    });

    Ok(())
}

pub fn send_global_state_updates(
    global_state: &mut GlobalState,
    to_server: &Sender<IPCMessage>,
) -> Result<(), NodeError> {
    let json = serde_json::to_value(global_state).context(JsonParserSnafu)?;

    block_on(async {
        to_server
            .send(IPCMessage::Json(json! {{
                "action": "state/updateGlobalState",
                "payload": json
            }}))
            .await
    });

    Ok(())
}
