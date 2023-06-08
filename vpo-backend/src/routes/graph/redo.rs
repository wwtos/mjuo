use ipc::ipc_message::IpcMessage;
use node_engine::{
    global_state::GlobalState,
    state::{ActionInvalidation, GraphState},
};
use serde_json::Value;
use snafu::ResultExt;

use crate::{
    errors::{EngineError, NodeSnafu},
    routes::RouteReturn,
    util::send_graph_updates,
    Sender,
};

pub fn route(
    _msg: Value,
    to_server: &Sender<IpcMessage>,
    state: &mut GraphState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    let updates = state.redo().context(NodeSnafu)?;

    for update in &updates {
        if let ActionInvalidation::GraphReindexNeeded(index) | ActionInvalidation::GraphModified(index) = update {
            send_graph_updates(state, *index, to_server)?;
        }
    }

    Ok(Some(RouteReturn {
        engine_updates: state
            .invalidations_to_engine_updates(updates, global_state)
            .context(NodeSnafu)?,
        new_project: false,
    }))
}
