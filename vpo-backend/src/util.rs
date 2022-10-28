use async_std::{channel::Sender, task::block_on};
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::{JsonParserSnafu, NodeError},
    graph_manager::GraphIndex,
    socket_registry::SocketRegistry,
    state::NodeEngineState,
};
use serde_json::json;
use snafu::ResultExt;

pub fn send_graph_updates(
    state: &mut NodeEngineState,
    graph_index: GraphIndex,
    to_server: &Sender<IPCMessage>,
) -> Result<(), NodeError> {
    let graph = &mut state
        .get_graph_manager()
        .get_graph_wrapper_mut(graph_index)
        .ok_or(NodeError::GraphDoesNotExist {
            graph_index: graph_index,
        })?
        .graph;

    let json = graph.serialize_to_json().unwrap();

    block_on(async {
        to_server
            .send(IPCMessage::Json(json! {{
                "action": "graph/updateGraph",
                "payload": {
                    "nodes": json["nodes"],
                    "connections": json["connections"],
                    "graphIndex": graph_index
                }
            }}))
            .await
    })
    .unwrap();

    Ok(())
}

pub fn send_registry_updates(registry: &mut SocketRegistry, to_server: &Sender<IPCMessage>) -> Result<(), NodeError> {
    let json = serde_json::to_value(registry).context(JsonParserSnafu)?;

    block_on(async {
        to_server
            .send(IPCMessage::Json(json! {{
                "action": "registry/updateRegistry",
                "payload": json
            }}))
            .await
    })
    .unwrap();

    Ok(())
}
