use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
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
    _current_graph_index: GraphIndex,
    graph_manager: &mut GraphManager,
    to_server: &Sender<IPCMessage>,
    _sound_config: &SoundConfig,
    _socket_registry: &mut SocketRegistry,
    _scripting_engine: &Engine,
) -> Result<Option<RouteReturn>, NodeError> {
    let new_graph_index = message["payload"].as_i64().unwrap() as u64;
    let graph = &graph_manager.get_graph_wrapper_ref(new_graph_index).unwrap().graph;

    update_graph(graph, new_graph_index, to_server);

    Ok(Some(RouteReturn {
        should_reindex_graph: false,
        new_graph_index: Some(new_graph_index),
    }))
}
