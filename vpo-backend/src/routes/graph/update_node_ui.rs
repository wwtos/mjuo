use ipc::ipc_message::IpcMessage;
use node_engine::{global_state::GlobalState, graph_manager::GraphIndex, node::NodeIndex, state::GraphState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::ResultExt;

use crate::{
    errors::{EngineError, JsonParserSnafu, NodeSnafu},
    routes::RouteReturn,
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
    _to_server: &Sender<IpcMessage>,
    state: &mut GraphState,
    _global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    let payload: Payload = serde_json::from_value(msg["payload"].take()).context(JsonParserSnafu)?;

    for (mut node_json, index) in payload.updated_nodes {
        if let Value::Object(ui_data) = node_json["uiData"].take() {
            let graph = state
                .get_graph_manager()
                .get_graph_mut(payload.graph_index)
                .context(NodeSnafu)?;

            let node = graph.get_node_mut(index).context(NodeSnafu)?;
            node.set_ui_data(ui_data.into_iter().collect());
        }
    }

    Ok(None)
}
