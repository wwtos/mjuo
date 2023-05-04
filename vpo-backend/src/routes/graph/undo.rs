use ipc::ipc_message::IpcMessage;
use node_engine::{global_state::GlobalState, state::NodeState};
use serde_json::Value;
use snafu::ResultExt;

use crate::{
    errors::{EngineError, NodeSnafu},
    routes::RouteReturn,
    util::send_graph_updates,
    Sender,
};

pub async fn route(
    _msg: Value,
    to_server: &Sender<IpcMessage>,
    state: &mut NodeState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    let (graphs_changed, _, traverser) = state.undo(global_state).context(NodeSnafu)?;

    for graph_index in graphs_changed {
        send_graph_updates(state, graph_index, to_server)?;
    }

    Ok(Some(RouteReturn {
        new_traverser: traverser,
        graph_operated_on: None,
        graph_to_reindex: None,
    }))
}
