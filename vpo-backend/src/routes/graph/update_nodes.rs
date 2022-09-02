use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::NodeError,
    graph_manager::GlobalNodeIndex,
    node::NodeIndex,
    state::{Action, ActionBundle, StateManager},
};
use serde_json::{Map, Value};

use crate::{
    util::{send_graph_updates, send_registry_updates},
    RouteReturn,
};

pub fn route(
    msg: Map<String, Value>,
    to_server: &Sender<IPCMessage>,
    state: &mut StateManager,
) -> Result<Option<RouteReturn>, NodeError> {
    let nodes_to_update = msg["payload"]["updatedNodes"]
        .as_array()
        .ok_or(NodeError::PropertyMissingOrMalformed("updatedNodes".to_string()))?;

    let graph_index = msg["payload"]["graphIndex"]
        .as_u64()
        .ok_or(NodeError::PropertyMissingOrMalformed("graphIndex".to_string()))?;

    let actions =
        nodes_to_update
            .iter()
            .try_fold(Vec::new(), |mut actions, node_json| -> Result<Vec<Action>, NodeError> {
                let index: NodeIndex = serde_json::from_value(node_json["index"].clone())?;

                if node_json["properties"].is_object() {
                    actions.push(Action::ChangeNodeProperties {
                        index: GlobalNodeIndex {
                            node_index: index,
                            graph_index: graph_index,
                        },
                        before: None,
                        after: serde_json::from_value(node_json["properties"].clone())?,
                    })
                }

                if node_json["ui_data"].is_object() {
                    actions.push(Action::ChangeNodeUiData {
                        index: GlobalNodeIndex {
                            node_index: index,
                            graph_index: graph_index,
                        },
                        before: None,
                        after: serde_json::from_value(node_json["ui_data"].clone())?,
                    })
                }

                if node_json["default_overrides"].is_array() {
                    actions.push(Action::ChangeNodeOverrides {
                        index: GlobalNodeIndex {
                            node_index: index,
                            graph_index: graph_index,
                        },
                        before: None,
                        after: serde_json::from_value(node_json["properties"].clone())?,
                    })
                }

                Ok(actions)
            })?;

    state.commit(ActionBundle::new(actions))?;

    send_graph_updates(state, graph_index, to_server)?;
    send_registry_updates(state.get_registry(), to_server)?;

    Ok(None)
}
