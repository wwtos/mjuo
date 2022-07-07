use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{errors::NodeError, node_graph::NodeGraph, node::NodeIndex, socket_registry::SocketRegistry};
use rhai::Engine;
use serde_json::{Map, Value};
use sound_engine::SoundConfig;

use crate::{RouteReturn, util::{update_graph, update_registry}};

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

            let did_apply_json = if let Some(generational_node) = graph.get_node(&index) {
                let mut node = (*generational_node.node).borrow_mut();

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
