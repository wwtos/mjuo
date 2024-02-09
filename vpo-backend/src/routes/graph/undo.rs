use std::collections::HashSet;

use node_engine::{graph_manager::GlobalNodeIndex, state::ActionInvalidation};
use snafu::ResultExt;

use crate::{
    errors::{EngineError, NodeSnafu},
    routes::prelude::*,
    routes::RouteReturn,
    util::{send_graph_updates, send_project_state_updates},
};

pub fn route(state: RouteState) -> Result<RouteReturn, EngineError> {
    let invalidations = state.state.undo().context(NodeSnafu)?;

    let mut touched_graphs = HashSet::new();

    for invalidation in &invalidations {
        match invalidation {
            ActionInvalidation::GraphReindexNeeded(index)
            | ActionInvalidation::GraphModified(index)
            | ActionInvalidation::NewDefaults(GlobalNodeIndex { graph_index: index, .. }, _)
            | ActionInvalidation::NewNode(GlobalNodeIndex { graph_index: index, .. }) => {
                touched_graphs.insert(index);
            }
            ActionInvalidation::NewRouteRules { .. } => {}
            ActionInvalidation::None => {}
        }
    }

    for graph_index in touched_graphs {
        send_graph_updates(state.state, *graph_index, state.to_server)?;
    }

    send_project_state_updates(state.state, state.global_state, state.to_server)?;
    state_invalidations(
        state.state,
        invalidations,
        &mut state.global_state.device_manager,
        &*state.resources_lock.read().unwrap(),
        state.to_audio_thread,
        state.to_server,
    )?;

    Ok(RouteReturn { new_project: false })
}
