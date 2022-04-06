use async_std::{channel::Sender};
use ipc::ipc_message::IPCMessage;
use node_engine::{graph::Graph, errors::NodeError, node::NodeIndex};
use serde_json::{Value, Map};
use sound_engine::SoundConfig;

use crate::{RouteReturn};

pub fn route(
    message: Map<String, Value>,
    graph: &mut Graph,
    _to_server: &Sender<IPCMessage>,
    _config: &SoundConfig
) -> Result<Option<RouteReturn>, NodeError> {
    let nodes_raw = message.get("payload").unwrap();

    if let Value::Array(nodes_to_update) = nodes_raw {
        for node_json in nodes_to_update {
            let index: NodeIndex =
                serde_json::from_value(node_json["index"].clone())?;

            let did_apply_json =
                if let Some(generational_node) = graph.get_node(&index) {
                    let mut node = (*generational_node.node).borrow_mut();

                    node.apply_json(node_json)?;

                    true
                } else {
                    false
                };

            if did_apply_json {
                graph.init_node(&index)?;
            }
        }
    }

    Ok(None)
}