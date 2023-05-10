use std::path::Path;

use ipc::ipc_message::IpcMessage;
use node_engine::{
    global_state::GlobalState,
    state::{NodeEngineUpdate, NodeState},
};
use rfd::AsyncFileDialog;
use serde_json::Value;
use snafu::ResultExt;

use crate::{
    errors::EngineError,
    io::load,
    routes::RouteReturn,
    util::{send_global_state_updates, send_graph_updates, send_registry_updates},
    Sender,
};

pub async fn route(
    _msg: Value,
    to_server: &Sender<IpcMessage>,
    state: &mut NodeState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    let file = AsyncFileDialog::new().pick_file().await;

    if let Some(file) = file {
        let path = file.path();

        global_state.active_project = Some(path.into());

        state.clear_history();
        load(Path::new(path), state, global_state)?;

        send_global_state_updates(global_state, to_server)?;
        send_registry_updates(state.get_registry(), to_server)?;
        send_graph_updates(state, state.get_root_graph_index(), to_server)?;

        return Ok(Some(RouteReturn {
            engine_updates: vec![NodeEngineUpdate::NewNodeEngine(
                state
                    .get_engine(global_state)
                    .whatever_context("could not create traverser")?,
            )],
            new_project: true,
        }));
    }

    Ok(None)
}
