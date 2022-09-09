use std::{error::Error, io::Write};

use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{errors::NodeError, graph_manager::GraphIndex, state::NodeEngineState};
use serde_json::Value;
use sound_engine::constants::BUFFER_SIZE;
use state::GlobalState;

pub mod io;
pub mod routes;
pub mod state;
pub mod util;

#[derive(Default)]
pub struct RouteReturn {
    pub graph_to_reindex: Option<GraphIndex>,
    pub graph_operated_on: Option<GraphIndex>,
}

pub fn route(
    msg: IPCMessage,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, NodeError> {
    let IPCMessage::Json(json) = msg;

    if let Value::Object(ref message) = json {
        let action = &message["action"];

        if let Value::String(action_name) = action {
            return match action_name.as_str() {
                "graph/get" => routes::graph::get::route(json, to_server, state, global_state),
                "graph/newNode" => routes::graph::new_node::route(json, to_server, state, global_state),
                "graph/removeNode" => routes::graph::remove_node::route(json, to_server, state, global_state),
                "graph/updateNodes" => routes::graph::update_nodes::route(json, to_server, state, global_state),
                "graph/updateNodesUi" => routes::graph::update_node_ui::route(json, to_server, state, global_state),
                "graph/connectNode" => routes::graph::connect_node::route(json, to_server, state, global_state),
                "graph/disconnectNode" => routes::graph::disconnect_node::route(json, to_server, state, global_state),
                "graph/undo" => routes::graph::undo::route(json, to_server, state, global_state),
                "graph/redo" => routes::graph::redo::route(json, to_server, state, global_state),
                _ => Ok(None),
            };
        }
    }

    Ok(None)
}

pub fn write_to_file(output_file: &mut std::fs::File, data: &[f32]) -> Result<(), Box<dyn Error>> {
    let mut data_out = [0_u8; BUFFER_SIZE * 4];

    // TODO: would memcpy work here faster?
    for i in 0..BUFFER_SIZE {
        let num = (data[i] as f32).to_le_bytes();

        data_out[i * 4] = num[0];
        data_out[i * 4 + 1] = num[1];
        data_out[i * 4 + 2] = num[2];
        data_out[i * 4 + 3] = num[3];
    }

    output_file.write_all(&data_out)?;

    Ok(())
}
