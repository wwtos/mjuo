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

use std::{error::Error, io::Write, thread};

#[cfg(any(unix, windows))]
use async_std::channel::{unbounded, Receiver};
use futures::executor::block_on;

#[cfg(target_arch = "wasm32")]
use ipc::send_buffer::SendBuffer;
#[cfg(any(unix, windows))]
use ipc::{ipc_message::IPCMessage, ipc_server::IPCServer};

use node_engine::{global_state::GlobalState, state::NodeState};
use routes::{route, RouteReturn};
use serde_json::json;

#[cfg(any(unix, windows))]
pub fn start_ipc() -> (Sender<IPCMessage>, Receiver<IPCMessage>) {
    let (to_server, from_main) = unbounded::<IPCMessage>();
    let (to_main, from_server) = unbounded::<IPCMessage>();

    let to_server_cloned = to_server.clone();

    thread::spawn(move || {
        IPCServer::open(to_server_cloned.clone(), from_main, to_main);
    });

    (to_server, from_server)
}

#[cfg(any(unix, windows))]
pub fn handle_msg(
    msg: IPCMessage,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeState,
    global_state: &mut GlobalState,
) {
    println!("got: {:?}", msg);
    let result = route(msg, to_server, state, global_state);

    match result {
        Ok(route_result) => {
            match route_result {
                Some(route_result) => route_result,
                None => RouteReturn::default(),
            };
        }
        Err(err) => {
            let err_str = err.to_string();

            block_on(async {
                to_server
                    .send(IPCMessage::Json(json! {{
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
        let num = (data[i] as f32).to_le_bytes();

        data_out[i * 4] = num[0];
        data_out[i * 4 + 1] = num[1];
        data_out[i * 4 + 2] = num[2];
        data_out[i * 4 + 3] = num[3];
    }

    output_file.write_all(&data_out)?;

    Ok(())
}
