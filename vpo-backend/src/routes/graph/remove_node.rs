use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::NodeError,
    graph_manager::GlobalNodeIndex,
    node::NodeIndex,
    state::{Action, ActionBundle, NodeEngineState},
};
use serde_json::Value;

use crate::{util::send_graph_updates, RouteReturn};

/// this function creates a new node in the graph based on the provided data
///
/// JSON should be formatted thus:
/// ```json
/// {
///     "action": "graph/deleteNode",
///     "payload": {
///         graphIndex: number,
///         nodeIndex: {
///             
///         }
///     }
/// }```
///
pub fn route(
    msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
) -> Result<Option<RouteReturn>, NodeError> {
    let node_index: NodeIndex = serde_json::from_value(msg["payload"]["nodeIndex"].clone())
        .map_err(|err| NodeError::JsonParserErrorInContext(err, "payload.nodeIndex".to_string()))?;

    let graph_index = msg["payload"]["graphIndex"]
        .as_u64()
        .ok_or(NodeError::PropertyMissingOrMalformed("payload.graphIndex".to_string()))?;

    state.commit(ActionBundle::new(vec![Action::RemoveNode {
        node_type: None,
        index: GlobalNodeIndex {
            node_index: node_index,
            graph_index: graph_index,
        },
        connections: None,
        serialized: None,
        child_graph_index: None,
        child_graph_io_indexes: None,
    }]))?;

    send_graph_updates(state, graph_index, to_server)?;

    Ok(Some(RouteReturn {
        graph_to_reindex: Some(graph_index),
        graph_operated_on: Some(graph_index),
    }))
}
