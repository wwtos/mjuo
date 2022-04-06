use async_std::{channel::Sender};
use ipc::ipc_message::IPCMessage;
use node_engine::{graph::Graph, errors::NodeError, connection::Connection};
use serde_json::{Value, Map};
use sound_engine::SoundConfig;

use crate::{RouteReturn, util::update_graph};

pub fn route(
    message: Map<String, Value>,
    graph: &mut Graph,
    to_server: &Sender<IPCMessage>,
    _config: &SoundConfig
) -> Result<Option<RouteReturn>, NodeError> {
    if let Value::Object(_) = &message["payload"] {
        let connection: Connection =
            serde_json::from_value(message["payload"].clone())?;

        graph.disconnect(
            connection.from_node,
            connection.from_socket_type,
            connection.to_node,
            connection.to_socket_type,
        )?;
    }

    update_graph(graph, to_server);

    Ok(Some(RouteReturn {
        should_reindex_graph: true
    }))
}