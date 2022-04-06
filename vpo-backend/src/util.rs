use async_std::{channel::Sender, task::block_on};
use ipc::ipc_message::IPCMessage;
use node_engine::graph::Graph;
use serde_json::json;

pub fn update_graph(graph: &Graph, to_server: &Sender<IPCMessage>) {
    let json = graph.serialize().unwrap();

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
