use std::collections::BTreeMap;

use ipc::ipc_message::IpcMessage;
use node_engine::{graph_manager::GraphIndex, node_graph::NodeConnectionData, node_wrapper::NodeWrapper};
use petgraph::Graph;
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

    let mut mini_graph: Graph<NodeWrapper, NodeConnectionData> = Graph::new();
    let mut mapping = BTreeMap::new();

    // populate the mini graph with all the selected nodes
    for (index, wrapper) in selected {
        let mut wrapper_cloned = wrapper.clone();

        // don't serialize the node rows
        wrapper_cloned.set_node_rows(vec![]);

        mapping.insert(index, mini_graph.add_node(wrapper_cloned));
    }

    // now transfer over the connections
    for (_, connection) in graph.edges_iter() {
        if let (Some(from), Some(to)) = (
            mapping.get(&node::NodeIndex(connection.get_from())),
            mapping.get(&node::NodeIndex(connection.get_to())),
        ) {
            mini_graph.add_edge(*from, *to, connection.data.clone());
        }
    }

    let graph_json = serde_json::to_string(&mini_graph).unwrap();

    let _ = state.to_server.send(IpcMessage::Json(json!({
        "action": "clipboard/set",
        "payload": graph_json,
    })));

    Ok(RouteReturn::default())
}
