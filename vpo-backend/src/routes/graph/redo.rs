use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{errors::NodeError, state::StateManager};
use serde_json::Value;

use crate::{util::send_graph_updates, RouteReturn};

pub fn route(
    _msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut StateManager,
) -> Result<Option<RouteReturn>, NodeError> {
    println!("redo");
    let graphs_changed = state.redo()?;

    for graph_index in graphs_changed {
        send_graph_updates(state, graph_index, to_server)?;
    }

    Ok(None)
}
