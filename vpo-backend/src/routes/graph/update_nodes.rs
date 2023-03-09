use std::collections::HashMap;

use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::{JsonParserSnafu, NodeError},
    global_state::GlobalState,
    graph_manager::{GlobalNodeIndex, GraphIndex},
    node::{NodeIndex, NodeRow},
    property::Property,
    state::{Action, ActionBundle, NodeEngineState},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::ResultExt;

use crate::{
    routes::RouteReturn,
    util::{send_graph_updates, send_registry_updates},
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

pub fn route(
    mut msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    let payload: Payload = serde_json::from_value(msg["payload"].take()).context(JsonParserSnafu)?;

    let actions = payload
        .updated_nodes
        .into_iter()
        .map(|(updated_node, index)| {
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
                        overrides: overrides,
                    }),
            ]
        })
        .flatten()
        .filter_map(|action| action)
        .collect();

    state.commit(ActionBundle::new(actions), global_state)?;

    send_graph_updates(state, payload.graph_index, to_server)?;
    send_registry_updates(state.get_registry(), to_server)?;

    Ok(None)
}
