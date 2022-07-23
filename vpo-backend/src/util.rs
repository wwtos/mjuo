use async_std::{channel::Sender, task::block_on};
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::NodeError, graph_manager::GraphIndex, node_graph::NodeGraph, socket_registry::SocketRegistry,
};
use serde_json::json;

pub fn update_graph(graph: &NodeGraph, graph_index: GraphIndex, to_server: &Sender<IPCMessage>) {
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
}

pub fn update_registry(registry: &mut SocketRegistry, to_server: &Sender<IPCMessage>) -> Result<(), NodeError> {
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
