use node_engine::{graph_manager::GraphIndex, node::NodeIndex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::ResultExt;

use crate::{
    errors::{EngineError, JsonParserSnafu, NodeSnafu},
    routes::prelude::*,
    routes::RouteReturn,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
    graph_index: GraphIndex,
    updated_nodes: Vec<(Value, NodeIndex)>,
}

pub fn route(mut state: RouteCtx) -> Result<RouteReturn, EngineError> {
    let payload: Payload = serde_json::from_value(state.msg["payload"].take()).context(JsonParserSnafu)?;

    for (mut node_json, index) in payload.updated_nodes {
        if let Value::Object(ui_data) = node_json["uiData"].take() {
            let graph = state
                .state
                .get_graph_manager()
                .get_graph_mut(payload.graph_index)
                .context(NodeSnafu)?;

            let node = graph.get_node_mut(index).context(NodeSnafu)?;
            node.set_ui_data(ui_data.into_iter().collect());
        }
    }

    Ok(RouteReturn::default())
}
