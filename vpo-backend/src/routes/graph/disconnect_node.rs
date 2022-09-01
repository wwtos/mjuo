use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::NodeError,
    state::{ActionBundle, Action, StateManager},
};
use serde_json::{Map, Value};

use crate::{util::send_graph_updates, RouteReturn};

pub fn route(
    msg: Map<String, Value>,
    to_server: &Sender<IPCMessage>,
    state: &mut StateManager,
) -> Result<Option<RouteReturn>, NodeError> {
    let graph_index = msg["payload"]["graphIndex"]
        .as_u64()
        .ok_or(NodeError::PropertyMissingOrMalformed("graphIndex".to_string()))?;

    state.commit(ActionBundle {
        actions: vec![
            Action::RemoveConnection {
                graph_index: graph_index,
                connection: serde_json::from_value(msg["payload"].clone())?
            },
        ]
    })?;

    send_graph_updates(state, graph_index, to_server);

    Ok(Some(RouteReturn {
        graph_to_reindex: Some(graph_index),
        new_graph_index: None,
    }))
}
