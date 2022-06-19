use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{errors::NodeError, graph::Graph};
use serde_json::{Map, Value};
use sound_engine::SoundConfig;

use crate::{util::update_graph, RouteReturn};

pub fn route(
    _message: Map<String, Value>,
    graph: &mut Graph,
    to_server: &Sender<IPCMessage>,
    _config: &SoundConfig,
) -> Result<Option<RouteReturn>, NodeError> {
    update_graph(graph, to_server);

    Ok(None)
}
