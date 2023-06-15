use node_engine::state::ActionInvalidation;
use snafu::ResultExt;

use crate::{
    errors::{EngineError, NodeSnafu},
    routes::prelude::*,
    routes::RouteReturn,
    util::send_graph_updates,
};

pub fn route(mut state: RouteState) -> Result<RouteReturn, EngineError> {
    let updates = state.state.redo().context(NodeSnafu)?;

    for update in &updates {
        if let ActionInvalidation::GraphReindexNeeded(index) | ActionInvalidation::GraphModified(index) = update {
            send_graph_updates(state.state, *index, state.to_server)?;
        }
    }

    Ok(RouteReturn {
        engine_updates: state
            .state
            .invalidations_to_engine_updates(updates, state.global_state)
            .context(NodeSnafu)?,
        new_project: false,
    })
}
