pub mod errors;
#[cfg(any(windows, unix))]
pub mod io;
pub mod migrations;
pub mod resource;
pub mod routes;
pub mod util;
#[cfg(target_arch = "wasm32")]
pub mod utils;
#[cfg(target_arch = "wasm32")]
pub mod wasm_lib;

#[cfg(target_arch = "wasm32")]
type Sender<T> = SendBuffer<T>;
#[cfg(any(unix, windows))]
type Sender<T> = broadcast::Sender<T>;

use std::path::PathBuf;
use std::sync::mpsc;
use std::{error::Error, io::Write};

#[cfg(any(unix, windows))]
use tokio::sync::broadcast;

use ipc::ipc_message::IpcMessage;
#[cfg(target_arch = "wasm32")]
use ipc::send_buffer::SendBuffer;

use io::file_watcher::FileWatcher;
use node_engine::{
    global_state::GlobalState,
    state::{GraphState, NodeEngineUpdate},
};
use routes::route;
use serde_json::json;

#[cfg(any(unix, windows))]
pub fn start_ipc(port: u32) -> (broadcast::Sender<IpcMessage>, broadcast::Receiver<IpcMessage>) {
    use ipc::ipc_server;

    let (to_tokio, _from_main) = broadcast::channel(16);
    let (to_main, from_tokio) = broadcast::channel(16);

    let to_tokio_cloned = to_tokio.clone();

    tokio::spawn(async move { ipc_server::start_ipc(to_tokio_cloned, to_main, port).await });

    (to_tokio, from_tokio)
}

#[cfg(any(unix, windows))]
pub async fn handle_msg(
    msg: IpcMessage,
    to_server: &broadcast::Sender<IpcMessage>,
    state: &mut GraphState,
    global_state: &mut GlobalState,
    engine_sender: &mpsc::Sender<Vec<NodeEngineUpdate>>,
    file_watcher: &mut FileWatcher,
    project_dir_sender: &mut broadcast::Sender<PathBuf>,
) {
    let result = route(msg, to_server, state, global_state).await;

    match result {
        Ok(Some(route_result)) => {
            if !route_result.engine_updates.is_empty() {
                engine_sender.send(route_result.engine_updates).unwrap();
            }

            if route_result.new_project {
                let _ = project_dir_sender.send(
                    global_state
                        .active_project
                        .as_ref()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .to_owned(),
                );

                file_watcher
                    .watch(global_state.active_project.as_ref().unwrap().parent().unwrap())
                    .unwrap();
            }
        }
        Ok(None) => {}
        Err(err) => {
            let err_str = err.to_string();

            to_server
                .send(IpcMessage::Json(json! {{
                    "action": "toast/error",
                    "payload": err_str
                }}))
                .unwrap();
        }
    }
}

#[cfg(any(unix, windows))]
pub fn write_to_file(output_file: &mut std::fs::File, data: &[f32]) -> Result<(), Box<dyn Error>> {
    let mut data_out = vec![0_u8; data.len() * 4];

    // TODO: would memcpy work here faster?
    for i in 0..data.len() {
        let num = data[i].to_le_bytes();

        data_out[i * 4] = num[0];
        data_out[i * 4 + 1] = num[1];
        data_out[i * 4 + 2] = num[2];
        data_out[i * 4 + 3] = num[3];
    }

    output_file.write_all(&data_out)?;

    Ok(())
}
