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
type Sender<T> = flume::Sender<T>;

use std::path::PathBuf;
use std::sync::RwLock;
use std::thread::JoinHandle;
use std::{error::Error, io::Write};

use ipc::ipc_message::IpcMessage;
#[cfg(target_arch = "wasm32")]
use ipc::send_buffer::SendBuffer;

use io::file_watcher::FileWatcher;
use node_engine::{
    global_state::{GlobalState, Resources},
    state::{GraphState, NodeEngineUpdate},
};
use routes::route;
use serde_json::json;

#[cfg(any(unix, windows))]
pub fn start_ipc(port: u32) -> (flume::Sender<IpcMessage>, flume::Receiver<IpcMessage>, JoinHandle<()>) {
    use ipc::ipc_server;

    let (to_server, from_main) = flume::unbounded();
    let (to_main, from_server) = flume::unbounded();

    let handle = ipc_server::start_ipc(from_main, to_main, port);

    (to_server, from_server, handle)
}

#[cfg(any(unix, windows))]
pub async fn handle_msg(
    msg: IpcMessage,
    to_server: &flume::Sender<IpcMessage>,
    state: &mut GraphState,
    global_state: &mut GlobalState,
    resources_lock: &RwLock<Resources>,
    engine_sender: &flume::Sender<Vec<NodeEngineUpdate>>,
    file_watcher: &mut FileWatcher,
    project_dir_sender: &mut flume::Sender<PathBuf>,
) {
    let result = route(msg, to_server, state, global_state, resources_lock).await;

    match result {
        Ok(route_result) => {
            if !route_result.engine_updates.is_empty() {
                engine_sender.send(route_result.engine_updates).unwrap();
            }

            if route_result.new_project {
                let _ = project_dir_sender.send(
                    global_state
                        .active_project
                        .as_ref()
                        .and_then(|x| x.parent())
                        .unwrap()
                        .to_owned(),
                );

                file_watcher
                    .watch(global_state.active_project.as_ref().unwrap().parent().unwrap())
                    .unwrap();
            }
        }
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
