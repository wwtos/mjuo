use std::collections::BTreeMap;

use ddgg::Graph;
use ipc::ipc_message::IpcMessage;
use node_engine::{graph_manager::GraphIndex, node_graph::NodeConnectionData, node_instance::NodeInstance};
use serde::Deserialize;
use serde_json::{json, Value};
use snafu::ResultExt;

use node_engine::node;

use crate::{
    errors::{JsonParserSnafu, NodeSnafu},
    routes::prelude::*,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
    graph_index: GraphIndex,
}

pub fn route(mut state: RouteState) -> Result<RouteReturn, EngineError> {
    let payload: Payload = serde_json::from_value(state.msg["payload"].take()).context(JsonParserSnafu)?;

    let graph = state
        .state
        .get_graph_manager()
        .get_graph(payload.graph_index)
        .context(NodeSnafu)?;

    let selected = graph
        .nodes_data_iter()
        .filter(|(_, node)| matches!(node.get_ui_data().get("selected"), Some(Value::Bool(true))));

    let mut mini_graph: Graph<Value, NodeConnectionData> = Graph::new();
    let mut mapping = BTreeMap::new();

    // populate the mini graph with all the selected nodes
    for (index, wrapper) in selected {
        let mut wrapper_json = json!(wrapper);

        // don't serialize all the properties
        if let Some(obj) = wrapper_json.as_object_mut() {
            obj.remove("nodeRows");
            obj.remove("state");
        }

        mapping.insert(index, mini_graph.add_vertex(wrapper_json).0);
    }

    // now transfer over the connections
    for (_, connection) in graph.edges_iter() {
        if let (Some(from), Some(to)) = (
            mapping.get(&node::NodeIndex(connection.get_from())),
            mapping.get(&node::NodeIndex(connection.get_to())),
        ) {
            mini_graph.add_edge(*from, *to, connection.data().clone()).unwrap();
        }
    }

    let graph_json = serde_json::to_string(&mini_graph).unwrap();

    let _ = state.to_server.send(IpcMessage::Json(json!({
        "action": "clipboard/set",
        "payload": graph_json,
    })));

    Ok(RouteReturn::default())
}
