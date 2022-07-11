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

use crate::{
    util::{update_graph, update_registry},
    RouteReturn,
};

pub fn route(
    _message: Map<String, Value>,
    current_graph_index: GraphIndex,
    graph_manager: &mut GraphManager,
    to_server: &Sender<IPCMessage>,
    _sound_config: &SoundConfig,
    socket_registry: &mut SocketRegistry,
    _scripting_engine: &Engine,
) -> Result<Option<RouteReturn>, NodeError> {
    let graph = &mut graph_manager.get_graph_wrapper_mut(current_graph_index).unwrap().graph;

    update_graph(graph, to_server);
    update_registry(socket_registry, to_server).unwrap();

    Ok(None)
}
