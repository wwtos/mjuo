use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::{JsonParserSnafu, NodeError},
    global_state::GlobalState,
    graph_manager::{GlobalNodeIndex, GraphIndex},
    node::NodeIndex,
    state::{Action, ActionBundle, NodeEngineState},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::ResultExt;

use crate::{routes::RouteReturn, util::send_graph_updates};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
    graph_index: GraphIndex,
    node_index: NodeIndex,
}

pub fn route(
    mut msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    let Payload {
        graph_index,
        node_index,
    } = serde_json::from_value(msg["payload"].take()).context(JsonParserSnafu)?;

    state.commit(
        ActionBundle::new(vec![Action::RemoveNode {
            index: GlobalNodeIndex {
                node_index,
                graph_index,
            },
        }]),
        global_state,
    )?;

    send_graph_updates(state, graph_index, to_server)?;

    Ok(Some(RouteReturn {
        graph_to_reindex: Some(graph_index),
        graph_operated_on: Some(graph_index),
    }))
}
