use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::{JsonParserErrorInContextSnafu, JsonParserSnafu, NodeError},
    global_state::GlobalState,
    graph_manager::{GlobalNodeIndex, GraphIndex},
    node::NodeIndex,
    state::{Action, ActionBundle, NodeEngineState},
};
use serde_json::Value;
use snafu::ResultExt;

use crate::{routes::RouteReturn, util::send_graph_updates};

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
    mut msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    let node_index: NodeIndex =
        serde_json::from_value(msg["payload"]["nodeIndex"].take()).context(JsonParserErrorInContextSnafu {
            context: "payload.nodeIndex".to_string(),
        })?;

    let graph_index: GraphIndex =
        serde_json::from_value(msg["payload"]["graphIndex"].take()).context(JsonParserSnafu)?;

    state.commit(
        ActionBundle::new(vec![Action::RemoveNode {
            index: GlobalNodeIndex {
                node_index,
                graph_index,
            },
        }]),
        global_state,
    )?;

    send_graph_updates(state, graph_index, to_server)?;

    Ok(Some(RouteReturn {
        graph_to_reindex: Some(graph_index),
        graph_operated_on: Some(graph_index),
    }))
}
