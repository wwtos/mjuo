use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::{JsonParserSnafu, NodeError},
    global_state::GlobalState,
    graph_manager::GraphIndex,
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
    mut msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    let ui_data = msg["payload"]["ui_data"].take();
    let graph_index: GraphIndex =
        serde_json::from_value(msg["payload"]["graphIndex"].take()).context(JsonParserSnafu)?;
    let node_type = msg["payload"]["type"]
        .as_str()
        .ok_or(NodeError::PropertyMissingOrMalformed {
            property_name: "payload.type".to_string(),
        })?;

    state.commit(
        ActionBundle::new(vec![Action::AddNode {
            node_type: node_type.to_string(),
            graph: graph_index,
        }]),
        global_state,
    )?;

    send_graph_updates(state, graph_index, to_server)?;

    Ok(Some(RouteReturn {
        graph_to_reindex: Some(graph_index),
        graph_operated_on: Some(graph_index),
    }))
}
