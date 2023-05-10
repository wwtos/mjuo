use ipc::ipc_message::IpcMessage;
use node_engine::{
    global_state::GlobalState,
    graph_manager::{GlobalNodeIndex, GraphIndex},
    node::NodeIndex,
    state::{Action, ActionBundle, NodeState},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::ResultExt;

use crate::{
    errors::{EngineError, JsonParserSnafu, NodeSnafu},
    routes::RouteReturn,
    util::send_graph_updates,
    Sender,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
    graph_index: GraphIndex,
    node_index: NodeIndex,
}

pub fn route(
    mut msg: Value,
    to_server: &Sender<IpcMessage>,
    state: &mut NodeState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    let Payload {
        graph_index,
        node_index,
    } = serde_json::from_value(msg["payload"].take()).context(JsonParserSnafu)?;

    let updates = state
        .commit(ActionBundle::new(vec![Action::RemoveNode {
            index: GlobalNodeIndex {
                node_index,
                graph_index,
            },
        }]))
        .context(NodeSnafu)?;

    send_graph_updates(state, graph_index, to_server)?;

    Ok(Some(RouteReturn {
        engine_updates: state.invalidations_to_engine_updates(updates, global_state),
        new_project: false,
    }))
}
