pub mod graph;
mod prelude;

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

pub struct RouteState<'a> {
    pub msg: Value,
    pub to_server: &'a Sender<IpcMessage>,
    pub state: &'a mut GraphState,
    pub global_state: &'a mut GlobalState,
}

pub async fn route(
    msg: IpcMessage,
    to_server: &Sender<IpcMessage>,
    state: &mut GraphState,
    global_state: &mut GlobalState,
) -> Result<RouteReturn, EngineError> {
    let IpcMessage::Json(json) = msg;

    if let Value::Object(ref message) = json {
        let action = &message["action"];

        if let Value::String(action_name) = action {
            let route_state = RouteState {
                msg: json.clone(),
                to_server,
                state,
                global_state,
            };

            return match action_name.as_str() {
                "graph/get" => graph::get::route(route_state),
                "graph/commit" => graph::commit::route(route_state),
                "graph/undo" => graph::undo::route(route_state),
                "graph/redo" => graph::redo::route(route_state),
                "graph/copy" => graph::copy::route(route_state),
                "graph/paste" => graph::paste::route(route_state),
                "graph/updateNodeUi" => graph::update_node_ui::route(route_state),
                "graph/updateNodeState" => graph::update_node_state::route(route_state),
                #[cfg(any(unix, windows))]
                "io/save" => io::save::route(route_state).await,
                #[cfg(any(unix, windows))]
                "io/load" => io::load::route(route_state).await,
                #[cfg(any(unix, windows))]
                "io/create" => io::create::route(route_state).await,
                #[cfg(any(unix, windows))]
                "io/importRank" => io::import_rank::route(route_state).await,
                _ => Ok(RouteReturn::default()),
            };
        }
    }

    Ok(RouteReturn::default())
}
