use async_std::{channel::Sender};
use ipc::ipc_message::IPCMessage;
use node_engine::{graph::Graph, errors::NodeError, nodes::variants::new_variant};
use serde_json::{Value, Map};
use sound_engine::SoundConfig;

use crate::{RouteReturn, util::update_graph};

/// this function creates a new node in the graph based on the provided data
/// 
/// JSON should be formatted thus:
/// ```json
/// {
///     "action": "graph/newNode",
///     "payload": {
///         "type": "[node type]",
///         "ui_data": {
///             foo: "override ui_data here"
///         }
///     }
/// }```
/// 
pub fn route(
    message: Map<String, Value>,
    graph: &mut Graph,
    to_server: &Sender<IPCMessage>,
    config: &SoundConfig
) -> Result<Option<RouteReturn>, NodeError> {
    let index = if let Value::String(node_type) = &message["payload"]["type"] {
        let new_node = new_variant(&node_type, config).unwrap();

        graph.add_node(new_node)
    } else {
        return Ok(None)
    };

    if let Value::Object(ui_data) = &message["payload"]["ui_data"] {
        let node_ref = graph.get_node(&index).unwrap().node;
        let mut node = (*node_ref).borrow_mut();

        // overwrite default values
        for (key, value) in ui_data.to_owned().into_iter() {
            node.set_ui_data_property(key, value);
        }
    }

    update_graph(graph, to_server);

    Ok(Some(RouteReturn {
        should_reindex_graph: true
    }))
}