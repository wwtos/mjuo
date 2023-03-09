use std::collections::HashMap;

use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::{JsonParserSnafu, NodeError},
    global_state::GlobalState,
    graph_manager::GraphIndex,
    state::{Action, ActionBundle, NodeEngineState},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::ResultExt;

use crate::{routes::RouteReturn, util::send_graph_updates};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
    node_type: String,
    ui_data: HashMap<String, Value>,
    graph_index: GraphIndex,
}

pub fn route(
    mut msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    let Payload {
        node_type,
        ui_data,
        graph_index,
    } = serde_json::from_value(msg["payload"].take()).context(JsonParserSnafu)?;

    let (_, created_nodes) = state.commit(
        ActionBundle::new(vec![Action::AddNode {
            node_type,
            graph: graph_index,
        }]),
        global_state,
    )?;

    let created = created_nodes.last().unwrap();

    let mut new_ui_data = state
        .get_graph_manager()
        .get_graph(created.graph_index)?
        .graph
        .borrow()
        .get_node(created.node_index)?
        .get_ui_data()
        .clone();

    new_ui_data.extend(ui_data);

    state.commit(
        ActionBundle::new(vec![Action::ChangeNodeUiData {
            index: *created,
            data: new_ui_data,
        }]),
        global_state,
    )?;

    send_graph_updates(state, graph_index, to_server)?;

    Ok(Some(RouteReturn {
        graph_to_reindex: Some(graph_index),
        graph_operated_on: Some(graph_index),
    }))
}
