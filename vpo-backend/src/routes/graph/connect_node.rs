use ipc::ipc_message::IpcMessage;
use node_engine::{
    connection::Connection,
    global_state::GlobalState,
    graph_manager::{GlobalNodeIndex, GraphIndex},
    node_graph::NodeConnection,
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
    to_server: &Sender<IpcMessage>,
    state: &mut NodeState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    let graph_index: GraphIndex =
        serde_json::from_value(msg["payload"]["graphIndex"].take()).context(JsonParserSnafu)?;
    let connection: Connection =
        serde_json::from_value(msg["payload"]["connection"].clone()).context(JsonParserSnafu)?;

    let (.., traverser) = state
        .commit(
            ActionBundle::new(vec![Action::ConnectNodes {
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
            }]),
            global_state,
        )
        .context(NodeSnafu)?;

    send_graph_updates(state, graph_index, to_server)?;

    Ok(Some(RouteReturn {
        graph_to_reindex: Some(graph_index),
        graph_operated_on: Some(graph_index),
        new_traverser: traverser,
        new_project: false,
    }))
}
