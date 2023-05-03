use std::collections::HashMap;

use ipc::ipc_message::IpcMessage;
use node_engine::{
    global_state::GlobalState,
    graph_manager::GraphIndex,
    state::{Action, ActionBundle, NodeState},
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
    state: &mut NodeState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    let Payload {
        node_type,
        ui_data,
        graph_index,
    } = serde_json::from_value(msg["payload"].take()).context(JsonParserSnafu)?;

    let (_, created_nodes, _) = state
        .commit(
            ActionBundle::new(vec![Action::AddNode {
                node_type,
                graph: graph_index,
            }]),
            global_state,
        )
        .context(NodeSnafu)?;

    let created = created_nodes.last().unwrap();

    let mut new_ui_data = state
        .get_graph_manager()
        .get_graph(created.graph_index)
        .context(NodeSnafu)?
        .graph
        .borrow()
        .get_node(created.node_index)
        .context(NodeSnafu)?
        .get_ui_data()
        .clone();

    new_ui_data.extend(ui_data);

    let (.., traverser) = state
        .commit(
            ActionBundle::new(vec![Action::ChangeNodeUiData {
                index: *created,
                data: new_ui_data,
            }]),
            global_state,
        )
        .context(NodeSnafu)?;

    send_registry_updates(state.get_registry(), to_server)?;
    send_graph_updates(state, graph_index, to_server)?;
    send_global_state_updates(global_state, to_server)?;

    Ok(Some(RouteReturn {
        graph_to_reindex: Some(graph_index),
        graph_operated_on: Some(graph_index),
        new_traverser: traverser,
    }))
}
