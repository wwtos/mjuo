use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    connection::Connection,
    errors::NodeError,
    graph_manager::{GraphIndex, GraphManager},
    socket_registry::SocketRegistry,
};
use rhai::Engine;
use serde_json::{Map, Value};
use sound_engine::SoundConfig;

use crate::{util::update_graph, RouteReturn};

pub fn route(
    message: Map<String, Value>,
    current_graph_index: GraphIndex,
    graph_manager: &mut GraphManager,
    to_server: &Sender<IPCMessage>,
    _sound_config: &SoundConfig,
    _socket_registry: &mut SocketRegistry,
    _scripting_engine: &Engine,
) -> Result<Option<RouteReturn>, NodeError> {
    let graph = &mut graph_manager.get_graph_wrapper_mut(current_graph_index).unwrap().graph;

    if let Value::Object(_) = &message["payload"] {
        let connection: Connection = serde_json::from_value(message["payload"].clone())?;

        graph.disconnect(
            &connection.from_node,
            &connection.from_socket_type,
            &connection.to_node,
            &connection.to_socket_type,
        )?;
    }

    update_graph(graph, current_graph_index, to_server);

    Ok(Some(RouteReturn {
        should_reindex_graph: true,
        new_graph_index: None,
    }))
}
