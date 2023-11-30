use std::collections::HashSet;

use node_engine::{graph_manager::GlobalNodeIndex, state::ActionInvalidation};
use snafu::ResultExt;

use crate::{
    errors::{EngineError, NodeSnafu},
    routes::prelude::*,
    routes::RouteReturn,
    util::send_graph_updates,
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
            ActionInvalidation::None => {}
        }
    }

    for graph_index in touched_graphs {
        send_graph_updates(state.state, *graph_index, state.to_server)?;
    }

    Ok(RouteReturn {
        engine_updates: state
            .state
            .invalidations_to_engine_updates(
                invalidations,
                state.global_state,
                &*state.resources_lock.read().unwrap(),
            )
            .context(NodeSnafu)?,
        new_project: false,
    })
}
