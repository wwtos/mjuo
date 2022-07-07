use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{connection::Connection, errors::NodeError, node_graph::NodeGraph};
use serde_json::{Map, Value};
use sound_engine::SoundConfig;

use crate::{util::update_graph, RouteReturn};

pub fn route(
    message: Map<String, Value>,
    graph: &mut NodeGraph,
    to_server: &Sender<IPCMessage>,
    _config: &SoundConfig,
) -> Result<Option<RouteReturn>, NodeError> {
    if let Value::Object(_) = &message["payload"] {
        let connection: Connection = serde_json::from_value(message["payload"].clone())?;

        graph.connect(
            connection.from_node,
            connection.from_socket_type,
            connection.to_node,
            connection.to_socket_type,
        )?;
    }

    update_graph(graph, to_server);

    Ok(Some(RouteReturn {
        should_reindex_graph: true,
    }))
}
