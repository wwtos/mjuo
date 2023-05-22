use std::collections::HashMap;

use ipc::ipc_message::IpcMessage;
use node_engine::{
    global_state::GlobalState,
    graph_manager::GraphIndex,
    state::{Action, ActionBundle, ActionInvalidation, GraphState},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::ResultExt;

use crate::{
    errors::{EngineError, JsonParserSnafu, NodeSnafu},
    routes::RouteReturn,
    util::{send_global_state_updates, send_graph_updates, send_registry_updates},
    Sender,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
    node_type: String,
    ui_data: HashMap<String, Value>,
    graph_index: GraphIndex,
}

pub fn route(
    mut msg: Value,
    to_server: &Sender<IpcMessage>,
    state: &mut GraphState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    let Payload {
        node_type,
        ui_data,
        graph_index,
    } = serde_json::from_value(msg["payload"].take()).context(JsonParserSnafu)?;

    let mut updates = state
        .commit(ActionBundle::new(vec![Action::AddNode {
            node_type,
            graph: graph_index,
        }]))
        .context(NodeSnafu)?;

    let created = updates
        .iter()
        .find_map(|update| match update {
            ActionInvalidation::NewNode(index) => Some(index),
            _ => None,
        })
        .unwrap();

    let mut new_ui_data = state
        .get_graph_manager()
        .get_graph(created.graph_index)
        .context(NodeSnafu)?
        .get_node(created.node_index)
        .context(NodeSnafu)?
        .get_ui_data()
        .clone();

    new_ui_data.extend(ui_data);

    let other_updates = state
        .commit(ActionBundle::new(vec![Action::ChangeNodeUiData {
            index: *created,
            data: new_ui_data,
        }]))
        .context(NodeSnafu)?;

    send_registry_updates(state.get_registry(), to_server)?;
    send_graph_updates(state, graph_index, to_server)?;
    send_global_state_updates(global_state, to_server)?;

    updates.extend(other_updates.into_iter());

    Ok(Some(RouteReturn {
        new_project: false,
        engine_updates: state.invalidations_to_engine_updates(updates, global_state),
    }))
}
