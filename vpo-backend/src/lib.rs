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

use std::sync::mpsc;
use std::{error::Error, io::Write};

#[cfg(any(unix, windows))]
use tokio::sync::broadcast;

use ipc::ipc_message::IpcMessage;
#[cfg(target_arch = "wasm32")]
use ipc::send_buffer::SendBuffer;

use node_engine::{engine::NodeEngine, global_state::GlobalState, state::NodeState};
use routes::route;
use serde_json::json;

#[cfg(any(unix, windows))]
pub async fn start_ipc() -> (broadcast::Sender<IpcMessage>, broadcast::Receiver<IpcMessage>) {
    use ipc::ipc_server;

    let (to_tokio, _from_main) = broadcast::channel(16);
    let (to_main, from_tokio) = broadcast::channel(16);

    let to_tokio_cloned = to_tokio.clone();

    tokio::spawn(async move { ipc_server::start_ipc(to_tokio_cloned, to_main).await });

    (to_tokio, from_tokio)
}

#[cfg(any(unix, windows))]
pub async fn handle_msg(
    msg: IpcMessage,
    to_server: &broadcast::Sender<IpcMessage>,
    state: &mut NodeState,
    global_state: &mut GlobalState,
    sender: &mpsc::Sender<NodeEngine>,
) {
    let result = route(msg, to_server, state, global_state).await;

    match result {
        Ok(route_result) => {
            if let Some(traverser) = route_result.and_then(|x| x.new_traverser) {
                let scripting_engine = rhai::Engine::new_raw();
                let (midi_in_node, output_node) = state.get_node_indexes();

                sender
                    .send(NodeEngine::new(
                        traverser,
                        scripting_engine,
                        midi_in_node,
                        output_node,
                        global_state.sound_config.clone(),
                    ))
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
