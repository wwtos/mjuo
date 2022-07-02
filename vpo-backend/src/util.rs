use async_std::{channel::Sender, task::block_on};
use ipc::ipc_message::IPCMessage;
use node_engine::{graph::Graph, socket_registry::SocketRegistry, errors::NodeError};
use serde_json::json;

pub fn update_graph(graph: &Graph, to_server: &Sender<IPCMessage>) {
    let json = graph.serialize_to_json().unwrap();

    block_on(async {
        to_server
            .send(IPCMessage::Json(json! {{
                "action": "graph/updateGraph",
                "payload": json
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
