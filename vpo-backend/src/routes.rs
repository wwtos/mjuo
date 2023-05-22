pub mod graph;

#[cfg(any(windows, unix))]
pub mod io;

use ipc::ipc_message::IpcMessage;
use node_engine::{
    global_state::GlobalState,
    state::{GraphState, NodeEngineUpdate},
};
use serde_json::Value;

use crate::{errors::EngineError, Sender};
#[derive(Default)]
pub struct RouteReturn {
    pub engine_updates: Vec<NodeEngineUpdate>,
    pub new_project: bool,
}

pub async fn route(
    msg: IpcMessage,
    to_server: &Sender<IpcMessage>,
    state: &mut GraphState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    let IpcMessage::Json(json) = msg;

    if let Value::Object(ref message) = json {
        let action = &message["action"];

        if let Value::String(action_name) = action {
            return match action_name.as_str() {
                "graph/get" => graph::get::route(json, to_server, state, global_state),
                "graph/newNode" => graph::new_node::route(json, to_server, state, global_state),
                "graph/removeNode" => graph::remove_node::route(json, to_server, state, global_state),
                "graph/updateNodes" => graph::update_nodes::route(json, to_server, state, global_state),
                "graph/updateNodesUi" => graph::update_node_ui::route(json, to_server, state, global_state),
                "graph/connectNode" => graph::connect_node::route(json, to_server, state, global_state),
                "graph/disconnectNode" => graph::disconnect_node::route(json, to_server, state, global_state),
                "graph/undo" => graph::undo::route(json, to_server, state, global_state),
                "graph/redo" => graph::redo::route(json, to_server, state, global_state),
                #[cfg(any(unix, windows))]
                "io/save" => io::save::route(json, to_server, state, global_state).await,
                #[cfg(any(unix, windows))]
                "io/load" => io::load::route(json, to_server, state, global_state).await,
                #[cfg(any(unix, windows))]
                "io/create" => io::create::route(json, to_server, state, global_state).await,
                #[cfg(any(unix, windows))]
                "io/importRank" => io::import_rank::route(json, to_server, state, global_state).await,
                _ => Ok(None),
            };
        }
    }

    Ok(None)
}
