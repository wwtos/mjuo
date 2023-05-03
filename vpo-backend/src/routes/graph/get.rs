use ipc::ipc_message::IpcMessage;
use node_engine::{global_state::GlobalState, graph_manager::GraphIndex, state::NodeState};
use serde_json::Value;
use snafu::ResultExt;
use tokio::sync::broadcast;

use crate::{
    errors::{EngineError, JsonParserSnafu},
    routes::RouteReturn,
    util::{send_global_state_updates, send_graph_updates, send_registry_updates},
};

pub async fn route(
    mut msg: Value,
    to_server: &broadcast::Sender<IpcMessage>,
    state: &mut NodeState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    let graph_index: GraphIndex =
        serde_json::from_value(msg["payload"]["graphIndex"].take()).context(JsonParserSnafu)?;

    send_registry_updates(state.get_registry(), to_server)?;
    send_graph_updates(state, graph_index, to_server)?;
    send_global_state_updates(global_state, to_server)?;

    Ok(None)
}
