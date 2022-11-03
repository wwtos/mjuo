use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::{JsonParserErrorInContextSnafu, NodeError},
    global_state::GlobalState,
    graph_manager::GlobalNodeIndex,
    node::NodeIndex,
    state::{Action, ActionBundle, NodeEngineState},
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
    global_state: &mut GlobalState,
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

    let actions =
        nodes_to_update
            .iter()
            .try_fold(Vec::new(), |mut actions, node_json| -> Result<Vec<Action>, NodeError> {
                let index: NodeIndex =
                    serde_json::from_value(node_json["index"].clone()).context(JsonParserErrorInContextSnafu {
                        context: "payload.updatedNodes[x].index".to_string(),
                    })?;

                if node_json["properties"].is_object() {
                    actions.push(Action::ChangeNodeProperties {
                        index: GlobalNodeIndex {
                            node_index: index,
                            graph_index: graph_index,
                        },
                        before: None,
                        after: serde_json::from_value(node_json["properties"].clone()).context(
                            JsonParserErrorInContextSnafu {
                                context: "payload.updatedNodes[x].properties".to_string(),
                            },
                        )?,
                    })
                }

                if node_json["ui_data"].is_object() {
                    actions.push(Action::ChangeNodeUiData {
                        index: GlobalNodeIndex {
                            node_index: index,
                            graph_index: graph_index,
                        },
                        before: None,
                        after: serde_json::from_value(node_json["ui_data"].clone()).context(
                            JsonParserErrorInContextSnafu {
                                context: "payload.updatedNodes[x].ui_data".to_string(),
                            },
                        )?,
                    })
                }

                if node_json["default_overrides"].is_array() {
                    actions.push(Action::ChangeNodeOverrides {
                        index: GlobalNodeIndex {
                            node_index: index,
                            graph_index: graph_index,
                        },
                        before: None,
                        after: serde_json::from_value(node_json["default_overrides"].clone()).context(
                            JsonParserErrorInContextSnafu {
                                context: "payload.updatedNodes[x].default_overrides".to_string(),
                            },
                        )?,
                    })
                }

                Ok(actions)
            })?;

    state.commit(ActionBundle::new(actions), global_state)?;

    send_graph_updates(state, graph_index, to_server)?;
    send_registry_updates(state.get_registry(), to_server)?;

    Ok(None)
}
