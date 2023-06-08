use ipc::ipc_message::IpcMessage;
use node_engine::{
    connection::Connection,
    global_state::GlobalState,
    graph_manager::{GlobalNodeIndex, GraphIndex},
    node_graph::NodeConnection,
    state::{Action, ActionBundle, GraphState},
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
    to_server: &Sender<IpcMessage>,
    state: &mut GraphState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    let graph_index: GraphIndex =
        serde_json::from_value(msg["payload"]["graphIndex"].take()).context(JsonParserSnafu)?;
    let connection: Connection =
        serde_json::from_value(msg["payload"]["connection"].clone()).context(JsonParserSnafu)?;

    let updates = state
        .commit(ActionBundle::new(vec![Action::ConnectNodes {
            from: GlobalNodeIndex {
                graph_index,
                node_index: connection.from_node,
            },
            to: GlobalNodeIndex {
                graph_index,
                node_index: connection.to_node,
            },
            data: NodeConnection {
                from_socket: connection.data.from_socket,
                to_socket: connection.data.to_socket,
            },
        }]))
        .context(NodeSnafu)?;

    send_graph_updates(state, graph_index, to_server)?;

    Ok(Some(RouteReturn {
        engine_updates: state
            .invalidations_to_engine_updates(updates, global_state)
            .context(NodeSnafu)?,
        new_project: false,
    }))
}
