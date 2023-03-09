use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    connection::Connection,
    errors::{JsonParserSnafu, NodeError},
    global_state::GlobalState,
    graph_manager::{GlobalNodeIndex, GraphIndex},
    node_graph::NodeConnection,
    state::{Action, ActionBundle, NodeEngineState},
};
use serde_json::Value;
use snafu::ResultExt;

use crate::{routes::RouteReturn, util::send_graph_updates};

pub fn route(
    mut msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    let graph_index: GraphIndex =
        serde_json::from_value(msg["payload"]["graphIndex"].take()).context(JsonParserSnafu)?;
    let connection: Connection =
        serde_json::from_value(msg["payload"]["connection"].clone()).context(JsonParserSnafu)?;

    state.commit(
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
                from_socket_type: connection.data.from_socket_type,
                to_socket_type: connection.data.to_socket_type,
            },
        }]),
        global_state,
    )?;

    send_graph_updates(state, graph_index, to_server)?;

    Ok(Some(RouteReturn {
        graph_to_reindex: Some(graph_index),
        graph_operated_on: Some(graph_index),
    }))
}
