use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::{JsonParserErrorInContextSnafu, NodeError},
    global_state::GlobalState,
    node::NodeIndex,
    state::NodeEngineState,
};
use serde_json::Value;
use snafu::ResultExt;

use crate::{
    routes::RouteReturn,
    util::{send_graph_updates, send_registry_updates},
};

pub fn route(
    msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    _global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    let nodes_to_update = msg["payload"]["updatedNodes"]
        .as_array()
        .ok_or(NodeError::PropertyMissingOrMalformed {
            property_name: "updatedNodes".to_string(),
        })?;

    let graph_index = msg["payload"]["graphIndex"]
        .as_u64()
        .ok_or(NodeError::PropertyMissingOrMalformed {
            property_name: "graphIndex".to_string(),
        })?;

    println!("{}", msg);

    for node_json in nodes_to_update {
        let index: NodeIndex =
            serde_json::from_value(node_json["index"].clone()).context(JsonParserErrorInContextSnafu {
                context: "payload.updatedNodes[x].index".to_string(),
            })?;

        if node_json["ui_data"].is_object() {
            let graph = &mut state.get_graph_manager().get_graph_wrapper_mut(graph_index).ok_or(
                NodeError::GraphDoesNotExist {
                    graph_index: graph_index,
                },
            )?;

            let node = graph
                .graph
                .get_node_mut(&index)
                .ok_or(NodeError::NodeDoesNotExist { node_index: index })?;

            node.apply_json(node_json)?;
        }
    }

    send_graph_updates(state, graph_index, to_server)?;
    send_registry_updates(state.get_registry(), to_server)?;

    Ok(None)
}
