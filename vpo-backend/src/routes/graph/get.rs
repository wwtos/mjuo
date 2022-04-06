use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{graph::Graph, errors::NodeError};
use serde_json::{Map, Value};
use sound_engine::SoundConfig;

use crate::{RouteReturn, util::update_graph};

pub fn route(
    _message: Map<String, Value>,
    graph: &mut Graph,
    to_server: &Sender<IPCMessage>,
    _config: &SoundConfig
) -> Result<Option<RouteReturn>, NodeError> {
    update_graph(graph, to_server);

    Ok(None)
}