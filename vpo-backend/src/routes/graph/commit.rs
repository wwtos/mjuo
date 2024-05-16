use std::collections::HashSet;

use node_engine::{graph_manager::GlobalNodeIndex, state::ActionInvalidation};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    errors::{JsonParserSnafu, NodeSnafu},
    routes::prelude::*,
    util::{send_graph_updates, send_project_state_updates},
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
    actions: ActionBundle,
    force_append: bool,
}

pub fn route(mut ctx: RouteCtx) -> Result<RouteReturn, EngineError> {
    let payload: Payload = serde_json::from_value(ctx.msg["payload"].take()).context(JsonParserSnafu)?;

    let invalidations = ctx
        .state
        .commit(payload.actions, payload.force_append)
        .context(NodeSnafu)?;

    let mut touched_graphs = HashSet::new();
    let mut new_route_rules = false;

    for invalidation in &invalidations {
        match invalidation {
            ActionInvalidation::GraphReindexNeeded(index)
            | ActionInvalidation::GraphModified(index)
            | ActionInvalidation::NewDefaults(GlobalNodeIndex { graph_index: index, .. }, _)
            | ActionInvalidation::NewNode(GlobalNodeIndex { graph_index: index, .. }) => {
                touched_graphs.insert(index);
            }
            ActionInvalidation::NewRouteRules { .. } => {
                new_route_rules = true;
            }
            ActionInvalidation::None => {}
        }
    }

    for graph_index in touched_graphs {
        send_graph_updates(ctx.state, *graph_index, ctx.to_server)?;
    }

    if new_route_rules {
        send_project_state_updates(ctx.state, ctx.global_state, ctx.to_server)?;
    }

    state_invalidations(
        ctx.state,
        invalidations,
        &mut ctx.global_state.device_manager,
        &*ctx.resources_lock.read().unwrap(),
        ctx.to_audio_thread,
        ctx.to_server,
    )?;

    Ok(RouteReturn { new_project: false })
}
