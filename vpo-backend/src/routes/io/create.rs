use ipc::ipc_message::IpcMessage;
use node_engine::{global_state::GlobalState, state::GraphState};
use rfd::AsyncFileDialog;
use serde_json::Value;

use crate::{errors::EngineError, io::save, routes::RouteReturn, Sender};

pub async fn route(
    _msg: Value,
    _to_server: &Sender<IpcMessage>,
    state: &mut GraphState,
    global_state: &mut GlobalState,
) -> Result<Option<RouteReturn>, EngineError> {
    let file = AsyncFileDialog::new().set_file_name("untitled.mjuo").save_file().await;

    if let Some(file) = file {
        save(state, file.path())?;

        global_state.active_project = Some(file.path().into());
    }

    Ok(None)
}
