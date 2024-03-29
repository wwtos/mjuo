use ipc::ipc_message::IpcMessage;
use node_engine::{graph_manager::GraphIndex, resources::Resources, state::GraphState};
use serde_json::json;
use snafu::ResultExt;

use crate::{
    errors::{EngineError, NodeSnafu},
    state::GlobalState,
    Sender,
};

pub fn send_graph_updates(
    state: &mut GraphState,
    graph_index: GraphIndex,
    to_server: &Sender<IpcMessage>,
) -> Result<(), EngineError> {
    let graph = state
        .get_graph_manager()
        .get_graph_mut(graph_index)
        .context(NodeSnafu)?;
    let json = serde_json::to_value(&*graph).unwrap();

    // we don't care if there's no receivers to state updates
    let _ = to_server.send(IpcMessage::Json(json! {{
        "action": "graph/updateGraph",
        "payload": {
            "nodes": json["nodes"],
            "graphIndex": graph_index
        }
    }}));

    Ok(())
}

pub fn send_project_state_updates(
    state: &GraphState,
    global_state: &GlobalState,
    to_server: &Sender<IpcMessage>,
) -> Result<(), EngineError> {
    let mut json = global_state.to_json();
    json["ioRoutes"] = json!(state.get_route_rules());

    let _ = to_server.send(IpcMessage::Json(json! {{
        "action": "state/updateState",
        "payload": json
    }}));

    Ok(())
}

pub fn send_resource_updates(resources: &Resources, to_server: &Sender<IpcMessage>) -> Result<(), EngineError> {
    let _ = to_server.send(IpcMessage::Json(json! {{
        "action": "state/updateResources",
        "payload": resources
    }}));

    Ok(())
}
