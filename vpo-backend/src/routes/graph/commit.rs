use std::collections::HashSet;

use node_engine::{graph_manager::GlobalNodeIndex, state::ActionInvalidation};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    errors::{JsonParserSnafu, NodeSnafu},
    routes::prelude::*,
    util::send_graph_updates,
};

#[derive(Serialize, Deserialize)]
struct Payload {
    actions: ActionBundle,
    force_append: bool,
}

pub fn route(mut state: RouteState) -> Result<RouteReturn, EngineError> {
    let payload: Payload = serde_json::from_value(state.msg["payload"].take()).context(JsonParserSnafu)?;

    let updates = state
        .state
        .commit(payload.actions, payload.force_append)
        .context(NodeSnafu)?;

    let mut touched_graphs = HashSet::new();

    for update in &updates {
        match update {
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
            .invalidations_to_engine_updates(updates, state.global_state)
            .context(NodeSnafu)?,
        new_project: false,
    })
}
