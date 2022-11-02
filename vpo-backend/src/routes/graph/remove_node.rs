use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::{JsonParserErrorInContextSnafu, NodeError},
    graph_manager::GlobalNodeIndex,
    node::NodeIndex,
    state::{Action, ActionBundle, AssetBundle, NodeEngineState},
};
use serde_json::Value;
use snafu::ResultExt;

use crate::{state::GlobalState, util::send_graph_updates, RouteReturn};

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
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    let node_index: NodeIndex =
        serde_json::from_value(msg["payload"]["nodeIndex"].clone()).context(JsonParserErrorInContextSnafu {
            context: "payload.nodeIndex".to_string(),
        })?;

    let graph_index = msg["payload"]["graphIndex"]
        .as_u64()
        .ok_or(NodeError::PropertyMissingOrMalformed {
            property_name: "payload.graphIndex".to_string(),
        })?;

    state.commit(
        ActionBundle::new(vec![Action::RemoveNode {
            node_type: None,
            index: GlobalNodeIndex {
                node_index: node_index,
                graph_index: graph_index,
            },
            connections: None,
            serialized: None,
            child_graph_index: None,
            child_graph_io_indexes: None,
        }]),
        AssetBundle {
            samples: &global_state.samples,
        },
    )?;

    send_graph_updates(state, graph_index, to_server)?;

    Ok(Some(RouteReturn {
        graph_to_reindex: Some(graph_index),
        graph_operated_on: Some(graph_index),
    }))
}
