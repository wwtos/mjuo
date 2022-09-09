use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::NodeError,
    state::{Action, ActionBundle, NodeEngineState},
};
use serde_json::Value;

use crate::{util::send_graph_updates, RouteReturn};

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
    msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
) -> Result<Option<RouteReturn>, NodeError> {
    let node_type = msg["payload"]["type"]
        .as_str()
        .ok_or(NodeError::PropertyMissingOrMalformed("payload.type".to_string()))?;
    let graph_index = msg["payload"]["graphIndex"]
        .as_u64()
        .ok_or(NodeError::PropertyMissingOrMalformed("payload.graphIndex".to_string()))?;

    state.commit(ActionBundle::new(vec![Action::CreateNode {
        node_type: node_type.to_string(),
        graph_index: graph_index,
        node_index: None,
        child_graph_index: None,
        child_graph_io_indexes: None,
    }]))?;

    // if let Value::Object(ui_data) = &message["payload"]["ui_data"] {
    //     let node = graph.get_node_mut(&index).unwrap();

    //     // overwrite default values
    //     for (key, value) in ui_data.to_owned().into_iter() {
    //         node.set_ui_data_property(key, value);
    //     }
    // }

    send_graph_updates(state, graph_index, to_server)?;

    Ok(Some(RouteReturn {
        graph_to_reindex: Some(graph_index),
        graph_operated_on: Some(graph_index),
    }))
}
