use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::NodeError,
    node::NodeIndex,
    state::{Action, ActionBundle, StateManager},
};
use serde_json::Value;

use crate::{
    util::{send_graph_updates, send_registry_updates},
    RouteReturn,
};

pub fn route(
    msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut StateManager,
) -> Result<Option<RouteReturn>, NodeError> {
    let nodes_to_update = msg["payload"]["updatedNodes"]
        .as_array()
        .ok_or(NodeError::PropertyMissingOrMalformed("updatedNodes".to_string()))?;

    let graph_index = msg["payload"]["graphIndex"]
        .as_u64()
        .ok_or(NodeError::PropertyMissingOrMalformed("graphIndex".to_string()))?;

    println!("{}", msg);

    let actions =
        nodes_to_update
            .iter()
            .try_fold(Vec::new(), |actions, node_json| -> Result<Vec<Action>, NodeError> {
                let index: NodeIndex = serde_json::from_value(node_json["index"].clone()).map_err(|err| {
                    NodeError::JsonParserErrorInContext(err, "payload.updatedNodes[x].index".to_string())
                })?;

                if node_json["ui_data"].is_object() {
                    let graph = &mut state
                        .get_graph_manager()
                        .get_graph_wrapper_mut(graph_index)
                        .ok_or(NodeError::GraphDoesNotExist(graph_index))?;

                    let node = graph
                        .graph
                        .get_node_mut(&index)
                        .ok_or(NodeError::NodeDoesNotExist(index))?;

                    node.apply_json(node_json)?;
                }

                Ok(actions)
            })?;

    state.commit(ActionBundle::new(actions))?;

    send_graph_updates(state, graph_index, to_server)?;
    send_registry_updates(state.get_registry(), to_server)?;

    Ok(None)
}
