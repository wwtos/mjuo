use node_engine::graph_manager::GraphIndex;
use snafu::ResultExt;

use crate::{
    errors::{EngineError, JsonParserSnafu},
    routes::prelude::*,
    routes::RouteReturn,
    util::{send_global_state_updates, send_graph_updates, send_registry_updates},
};

pub fn route(mut state: RouteState) -> Result<RouteReturn, EngineError> {
    let graph_index: GraphIndex =
        serde_json::from_value(state.msg["payload"]["graphIndex"].take()).context(JsonParserSnafu)?;

    send_registry_updates(state.state.get_registry(), state.to_server)?;
    send_graph_updates(state.state, graph_index, state.to_server)?;
    send_global_state_updates(state.global_state, state.to_server)?;

    Ok(RouteReturn::default())
}
