use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{errors::NodeError, node::NodeIndex, node_graph::NodeGraph, socket_registry::SocketRegistry};
use rhai::Engine;
use serde_json::{Map, Value};
use sound_engine::SoundConfig;

use crate::{
    util::{update_graph, update_registry},
    RouteReturn,
};

pub fn route(
    message: Map<String, Value>,
    graph: &mut NodeGraph,
    to_server: &Sender<IPCMessage>,
    _config: &SoundConfig,
    socket_registry: &mut SocketRegistry,
    scripting_engine: &Engine,
) -> Result<Option<RouteReturn>, NodeError> {
    let nodes_raw = message.get("payload").unwrap();

    let mut did_any_node_change = false;

    if let Value::Array(nodes_to_update) = nodes_raw {
        for node_json in nodes_to_update {
            let index: NodeIndex = serde_json::from_value(node_json["index"].clone())?;

            let did_apply_json = if let Some(node) = graph.get_node_mut(&index) {
                node.apply_json(node_json)?;

                true
            } else {
                false
            };

            if did_apply_json {
                if graph.init_node(&index, socket_registry, &scripting_engine)? {
                    did_any_node_change = true;
                }
            }
        }
    }

    if did_any_node_change {
        update_graph(graph, to_server);
        update_registry(socket_registry, to_server).unwrap();
    }

    Ok(None)
}
