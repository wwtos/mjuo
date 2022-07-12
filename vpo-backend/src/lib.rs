use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::NodeError,
    graph_manager::{GraphIndex, GraphManager},
    socket_registry::SocketRegistry,
};
use rhai::Engine;
use serde_json::Value;
use sound_engine::SoundConfig;

pub mod routes;
pub mod util;

#[derive(Default)]
pub struct RouteReturn {
    pub should_reindex_graph: bool,
    pub new_graph_index: Option<GraphIndex>
}

pub fn route(
    msg: IPCMessage,
    current_graph_index: GraphIndex,
    graph_manager: &mut GraphManager,
    to_server: &Sender<IPCMessage>,
    sound_config: &SoundConfig,
    socket_registry: &mut SocketRegistry,
    scripting_engine: &Engine,
) -> Result<Option<RouteReturn>, NodeError> {
    let IPCMessage::Json(json) = msg;

    if let Value::Object(message) = json {
        let action = message.get("action");

        if let Some(Value::String(action_name)) = action {
            return match action_name.as_str() {
                "graph/get" => routes::graph::get::route(
                    message,
                    current_graph_index,
                    graph_manager,
                    to_server,
                    sound_config,
                    socket_registry,
                    scripting_engine,
                ),
                "graph/newNode" => routes::graph::new_node::route(
                    message,
                    current_graph_index,
                    graph_manager,
                    to_server,
                    sound_config,
                    socket_registry,
                    scripting_engine,
                ),
                "graph/updateNodes" => routes::graph::update_nodes::route(
                    message,
                    current_graph_index,
                    graph_manager,
                    to_server,
                    sound_config,
                    socket_registry,
                    scripting_engine,
                ),
                "graph/connectNode" => routes::graph::connect_node::route(
                    message,
                    current_graph_index,
                    graph_manager,
                    to_server,
                    sound_config,
                    socket_registry,
                    scripting_engine,
                ),
                "graph/disconnectNode" => routes::graph::disconnect_node::route(
                    message,
                    current_graph_index,
                    graph_manager,
                    to_server,
                    sound_config,
                    socket_registry,
                    scripting_engine,
                ),
                "graph/switchGraph" => routes::graph::switch_graph::route(
                    message,
                    current_graph_index,
                    graph_manager,
                    to_server,
                    sound_config,
                    socket_registry,
                    scripting_engine,
                ),
                _ => Ok(None),
            };
        }
    }

    Ok(None)
}
