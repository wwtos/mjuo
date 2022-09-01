use async_std::{channel::Sender, task::block_on};
use ipc::ipc_message::IPCMessage;
use node_engine::{errors::NodeError, graph_manager::GraphIndex, socket_registry::SocketRegistry, state::StateManager};
use serde_json::json;

pub fn send_graph_updates(
    state: &StateManager,
    graph_index: GraphIndex,
    to_server: &Sender<IPCMessage>,
) -> Result<(), NodeError> {
    let graph = &mut state
        .get_graph_manager()
        .get_graph_wrapper_mut(graph_index)
        .ok_or(NodeError::GraphDoesNotExist(graph_index))?
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
    let json = serde_json::to_value(registry).map_err(|err| NodeError::JsonParserError(err))?;

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
