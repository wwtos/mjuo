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

#[cfg(any(unix, windows))]
type Sender<T> = async_std::channel::Sender<T>;
#[cfg(target_arch = "wasm32")]
type Sender<T> = SendBuffer<T>;

use std::sync::mpsc;
use std::{error::Error, io::Write, thread};

#[cfg(any(unix, windows))]
use async_std::channel::{unbounded, Receiver};
use futures::executor::block_on;

use ipc::ipc_message::IpcMessage;
#[cfg(target_arch = "wasm32")]
use ipc::send_buffer::SendBuffer;

use node_engine::{engine::NodeEngine, global_state::GlobalState, state::NodeState};
use routes::route;
use serde_json::json;

#[cfg(any(unix, windows))]
pub fn start_ipc() -> (Sender<IpcMessage>, Receiver<IpcMessage>) {
    use futures::join;
    use ipc::ipc_server;
    use tokio::{runtime::Runtime, sync::broadcast};

    let (to_tokio_sync, from_main_sync) = unbounded::<IpcMessage>();
    let (to_main_sync, from_tokio_sync) = unbounded::<IpcMessage>();

    thread::spawn(move || {
        let runtime = Runtime::new().unwrap();

        runtime.block_on(async {
            let (to_tokio, _from_main) = broadcast::channel(16);
            let (to_main, mut from_tokio) = broadcast::channel(16);

            join!(
                ipc_server::start_ipc(to_tokio.clone(), to_main),
                async {
                    loop {
                        let message = from_tokio.recv().await.unwrap();
                        to_main_sync.send(message).await.unwrap();
                    }
                },
                async move {
                    loop {
                        let message = from_main_sync.recv().await.unwrap();
                        to_tokio.send(message).unwrap();
                    }
                }
            );
        });
    });

    (to_tokio_sync, from_tokio_sync)
}

#[cfg(any(unix, windows))]
pub fn handle_msg(
    msg: IpcMessage,
    to_server: &Sender<IpcMessage>,
    state: &mut NodeState,
    global_state: &mut GlobalState,
    sender: &mpsc::Sender<NodeEngine>,
) {
    // println!("got: {:?}", msg);
    let result = route(msg, to_server, state, global_state);

    match result {
        Ok(route_result) => {
            if let Some(traverser) = route_result.and_then(|x| x.new_traverser) {
                let scripting_engine = rhai::Engine::new_raw();
                let (midi_in_node, output_node) = state.get_node_indexes();

                sender
                    .send(NodeEngine::new(traverser, scripting_engine, midi_in_node, output_node))
                    .unwrap();
            }
        }
        Err(err) => {
            let err_str = err.to_string();

            block_on(async {
                to_server
                    .send(IpcMessage::Json(json! {{
                        "action": "toast/error",
                        "payload": err_str
                    }}))
                    .await
            })
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
