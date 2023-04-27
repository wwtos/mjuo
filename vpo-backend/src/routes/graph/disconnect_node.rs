use ipc::ipc_message::IPCMessage;
use node_engine::{
    connection::Connection,
    global_state::GlobalState,
    graph_manager::{GlobalNodeIndex, GraphIndex},
    state::{Action, ActionBundle, NodeState},
};
use serde_json::Value;
use snafu::ResultExt;

use crate::{
    errors::{EngineError, JsonParserSnafu, NodeSnafu},
    routes::RouteReturn,
    util::send_graph_updates,
    Sender,
};

pub fn route(
    mut msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    let graph_index: GraphIndex =
        serde_json::from_value(msg["payload"]["graphIndex"].take()).context(JsonParserSnafu)?;
    let connection: Connection =
        serde_json::from_value(msg["payload"]["connection"].clone()).context(JsonParserSnafu)?;

    state
        .commit(
            ActionBundle::new(vec![Action::DisconnectNodes {
                from: GlobalNodeIndex {
                    graph_index,
                    node_index: connection.from_node,
                },
                to: GlobalNodeIndex {
                    graph_index,
                    node_index: connection.to_node,
                },
                data: connection.data,
            }]),
            global_state,
        )
        .context(NodeSnafu)?;

    send_graph_updates(state, graph_index, to_server)?;

    Ok(Some(RouteReturn {
        graph_to_reindex: Some(graph_index),
        graph_operated_on: Some(graph_index),
    }))
}
