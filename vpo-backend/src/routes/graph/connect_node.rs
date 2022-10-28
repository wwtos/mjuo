use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::{JsonParserSnafu, NodeError},
    state::{Action, ActionBundle, NodeEngineState},
};
use serde_json::Value;
use snafu::ResultExt;

use crate::{state::GlobalState, util::send_graph_updates, RouteReturn};

pub fn route(
    msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    _global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    let graph_index = msg["payload"]["graphIndex"]
        .as_u64()
        .ok_or(NodeError::PropertyMissingOrMalformed {
            property_name: "graphIndex".to_string(),
        })?;

    state.commit(ActionBundle::new(vec![Action::AddConnection {
        graph_index: graph_index,
        connection: serde_json::from_value(msg["payload"]["connection"].clone()).context(JsonParserSnafu)?,
    }]))?;

    send_graph_updates(state, graph_index, to_server)?;

    Ok(Some(RouteReturn {
        graph_to_reindex: Some(graph_index),
        graph_operated_on: Some(graph_index),
    }))
}
