use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::{JsonParserSnafu, NodeError},
    global_state::GlobalState,
    graph_manager::GraphIndex,
    node::NodeIndex,
    state::NodeEngineState,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::ResultExt;

use crate::{
    routes::RouteReturn,
    util::{send_graph_updates, send_registry_updates},
    Sender,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
    graph_index: GraphIndex,
    updated_nodes: Vec<(Value, NodeIndex)>,
}

pub fn route(
    mut msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    _global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    let payload: Payload = serde_json::from_value(msg["payload"].take()).context(JsonParserSnafu)?;

    for (mut node_json, index) in payload.updated_nodes {
        if node_json["uiData"].is_object() {
            let mut graph = state
                .get_graph_manager()
                .get_graph(payload.graph_index)?
                .graph
                .borrow_mut();

            let node = graph.get_node_mut(index)?;

            node.apply_json(&mut node_json)?;
        }
    }

    send_registry_updates(state.get_registry(), to_server)?;
    send_graph_updates(state, payload.graph_index, to_server)?;

    Ok(None)
}
