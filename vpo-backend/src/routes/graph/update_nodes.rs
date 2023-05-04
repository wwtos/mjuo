use ipc::ipc_message::IpcMessage;
use node_engine::{
    global_state::GlobalState,
    graph_manager::{GlobalNodeIndex, GraphIndex},
    node::{NodeIndex, NodeRow},
    property::Property,
    state::{Action, ActionBundle, NodeState},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::ResultExt;
use std::collections::HashMap;

use crate::{
    errors::{EngineError, JsonParserSnafu, NodeSnafu},
    routes::RouteReturn,
    util::{send_graph_updates, send_registry_updates},
    Sender,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiNodeWrapper {
    #[serde(default)]
    properties: Option<HashMap<String, Property>>,
    #[serde(default)]
    ui_data: Option<HashMap<String, Value>>,
    #[serde(default)]
    default_overrides: Option<Vec<NodeRow>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
    graph_index: GraphIndex,
    updated_nodes: Vec<(ApiNodeWrapper, NodeIndex)>,
}

pub async fn route(
    mut msg: Value,
    to_server: &Sender<IpcMessage>,
    state: &mut NodeState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    let payload: Payload = serde_json::from_value(msg["payload"].take()).context(JsonParserSnafu)?;

    let actions = payload
        .updated_nodes
        .into_iter()
        .flat_map(|(updated_node, index)| {
            [
                updated_node.properties.map(|properties| Action::ChangeNodeProperties {
                    index: GlobalNodeIndex {
                        node_index: index,
                        graph_index: payload.graph_index,
                    },
                    props: properties,
                }),
                updated_node.ui_data.map(|ui_data| Action::ChangeNodeUiData {
                    index: GlobalNodeIndex {
                        node_index: index,
                        graph_index: payload.graph_index,
                    },
                    data: ui_data,
                }),
                updated_node
                    .default_overrides
                    .map(|overrides| Action::ChangeNodeOverrides {
                        index: GlobalNodeIndex {
                            node_index: index,
                            graph_index: payload.graph_index,
                        },
                        overrides,
                    }),
            ]
        })
        .flatten()
        .collect();

    let (.., traverser) = state
        .commit(ActionBundle::new(actions), global_state)
        .context(NodeSnafu)?;

    send_registry_updates(state.get_registry(), to_server)?;
    send_graph_updates(state, payload.graph_index, to_server)?;

    Ok(Some(RouteReturn {
        new_traverser: traverser,
        graph_operated_on: None,
        graph_to_reindex: None,
    }))
}
