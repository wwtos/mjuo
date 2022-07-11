use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{errors::NodeError, node_graph::NodeGraph, socket_registry::SocketRegistry};
use serde_json::{Map, Value};
use sound_engine::SoundConfig;

use crate::{
    util::{update_graph, update_registry},
    RouteReturn,
};

pub fn route(
    _message: Map<String, Value>,
    graph: &mut NodeGraph,
    to_server: &Sender<IPCMessage>,
    _config: &SoundConfig,
    socket_registry: &mut SocketRegistry,
) -> Result<Option<RouteReturn>, NodeError> {
    update_graph(graph, to_server);
    update_registry(socket_registry, to_server).unwrap();

    Ok(None)
}
