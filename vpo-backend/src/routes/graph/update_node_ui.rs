use ipc::ipc_message::IpcMessage;
use node_engine::{global_state::GlobalState, graph_manager::GraphIndex, node::NodeIndex, state::NodeState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::ResultExt;

use crate::{
    errors::{EngineError, JsonParserSnafu, NodeSnafu},
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

pub async fn route(
    mut msg: Value,
    to_server: &Sender<IpcMessage>,
    state: &mut NodeState,
    _global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    let payload: Payload = serde_json::from_value(msg["payload"].take()).context(JsonParserSnafu)?;

    for (mut node_json, index) in payload.updated_nodes {
        if let Value::Object(ui_data) = node_json["uiData"].take() {
            let mut graph = state
                .get_graph_manager()
                .get_graph(payload.graph_index)
                .context(NodeSnafu)?
                .graph
                .borrow_mut();

            let node = graph.get_node_mut(index).context(NodeSnafu)?;
            node.set_ui_data(ui_data.into_iter().collect());
        }
    }

    send_registry_updates(state.get_registry(), to_server)?;
    send_graph_updates(state, payload.graph_index, to_server)?;

    Ok(None)
}
