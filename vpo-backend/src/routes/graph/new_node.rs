use std::collections::HashMap;

use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::NodeError,
    global_state::GlobalState,
    graph_manager::GlobalNodeIndex,
    state::{Action, ActionBundle, NodeEngineState},
};
use serde_json::Value;

use crate::{routes::RouteReturn, util::send_graph_updates};

/// this function creates a new node in the graph based on the provided data
///
/// JSON should be formatted thus:
/// ```json
/// {
///     "action": "graph/newNode",
///     "payload": {
///         "type": "[node type]",
///         "ui_data": {
///             foo: "override ui_data here"
///         }
///     }
/// }```
///
pub fn route(
    mut msg: Value,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    let ui_data = msg["payload"]["ui_data"].take();
    let node_type = msg["payload"]["type"]
        .as_str()
        .ok_or(NodeError::PropertyMissingOrMalformed {
            property_name: "payload.type".to_string(),
        })?;
    let graph_index = msg["payload"]["graphIndex"]
        .as_u64()
        .ok_or(NodeError::PropertyMissingOrMalformed {
            property_name: "payload.graphIndex".to_string(),
        })?;

    state.commit(
        ActionBundle::new(vec![Action::CreateNode {
            node_type: node_type.to_string(),
            graph_index: graph_index,
            node_index: None,
            child_graph_index: None,
            child_graph_io_indexes: None,
        }]),
        global_state,
    )?;

    // get the new index and apply the ui data
    if let Value::Object(new_ui_data) = ui_data {
        let history = state.get_history_ref();
        let last_action = history[history.len() - 1].actions[0].clone();

        match last_action {
            Action::CreateNode {
                graph_index,
                node_index,
                ..
            } => {
                let mut current_ui_data = state
                    .get_graph_manager()
                    .get_graph_wrapper_ref(graph_index)
                    .ok_or(NodeError::GraphDoesNotExist { graph_index })?
                    .graph
                    .get_node(&node_index.unwrap())
                    .ok_or(NodeError::NodeDoesNotExist {
                        node_index: node_index.unwrap(),
                    })?
                    .get_ui_data()
                    .clone();

                let new_ui_data = value_to_hashmap(new_ui_data);
                current_ui_data.extend(new_ui_data);

                state.commit(
                    ActionBundle::new(vec![Action::ChangeNodeUiData {
                        index: GlobalNodeIndex {
                            graph_index,
                            node_index: node_index.unwrap(),
                        },
                        before: None,
                        after: current_ui_data,
                    }]),
                    global_state,
                )?;
            }
            _ => {}
        }
    }
    // if let Value::Object(ui_data) = &message["payload"]["ui_data"] {
    //     let node = graph.get_node_mut(&index).unwrap();

    //     // overwrite default values
    //     for (key, value) in ui_data.to_owned().into_iter() {
    //         node.set_ui_data_property(key, value);
    //     }
    // }

    send_graph_updates(state, graph_index, to_server)?;

    Ok(Some(RouteReturn {
        graph_to_reindex: Some(graph_index),
        graph_operated_on: Some(graph_index),
    }))
}

fn value_to_hashmap(map: serde_json::Map<String, Value>) -> HashMap<String, Value> {
    map.into_iter().collect::<HashMap<String, Value>>()
}
