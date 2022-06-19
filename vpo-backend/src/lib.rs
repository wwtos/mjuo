use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{errors::NodeError, graph::Graph};
use serde_json::Value;
use sound_engine::SoundConfig;

pub mod routes;
pub mod util;

pub struct RouteReturn {
    pub should_reindex_graph: bool,
}

pub fn route(
    msg: IPCMessage,
    graph: &mut Graph,
    to_server: &Sender<IPCMessage>,
    config: &SoundConfig,
) -> Result<Option<RouteReturn>, NodeError> {
    let IPCMessage::Json(json) = msg;

    if let Value::Object(message) = json {
        let action = message.get("action");

        if let Some(Value::String(action_name)) = action {
            return match action_name.as_str() {
                "graph/get" => routes::graph::get::route(message, graph, to_server, config),
                "graph/newNode" => {
                    routes::graph::new_node::route(message, graph, to_server, config)
                }
                "graph/updateNodes" => {
                    routes::graph::update_nodes::route(message, graph, to_server, config)
                }
                "graph/connectNode" => {
                    routes::graph::connect_node::route(message, graph, to_server, config)
                }
                "graph/disconnectNode" => {
                    routes::graph::disconnect_node::route(message, graph, to_server, config)
                }
                _ => Ok(None),
            };
        }
    }

    Ok(None)
}
