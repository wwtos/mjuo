use std::thread::{self, JoinHandle};

use tokio::net::{TcpListener, TcpStream};
use tokio::runtime;

use futures::{try_join, SinkExt, StreamExt};
use log::info;
use snafu::ResultExt;
use tokio_tungstenite::tungstenite::Message;

use crate::error::{IpcError, WebsocketSnafu};
use crate::ipc_message::IpcMessage;

pub fn start_ipc(
    from_main: flume::Receiver<IpcMessage>,
    to_main: flume::Sender<IpcMessage>,
    port: u32,
) -> JoinHandle<()> {
    thread::spawn(move || {
        // let me manage my own dang threads, will you?
        let rt = runtime::Builder::new_current_thread().enable_io().build().unwrap();

        rt.block_on(async {
            // TODO: fail gracefully
            let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
                .await
                .expect("failed to bind");

            while let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(create_connection_task(stream, from_main.clone(), to_main.clone()));
            }
        });
    })
}

async fn create_connection_task(
    stream: TcpStream,
    mut from_main: flume::Receiver<IpcMessage>,
    to_main: flume::Sender<IpcMessage>,
) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    let (mut to_client, mut from_client) = ws_stream.split();

    info!("New WebSocket connection: {}", addr);

    let _ = try_join!(
        // from websocket to main
        async move {
            while let Some(msg) = from_client.next().await {
                let msg = msg.context(WebsocketSnafu)?;

                if let Message::Text(text) = msg {
                    if let Ok(json) = serde_json::from_str(&text) {
                        to_main.send(IpcMessage::Json(json)).unwrap();
                    }
                }
            }

            Ok::<(), IpcError>(())
        },
        // from main to websocket
        async move {
            loop {
                let msg = from_main.recv_async().await.unwrap();

                let IpcMessage::Json(json) = msg;

                let err = to_client
                    .send(Message::Text(serde_json::to_string(&json).unwrap()))
                    .await;

                if let Err(tokio_tungstenite::tungstenite::error::Error::ConnectionClosed) = err {
                    break;
                }
            }

            // for type annotations
            #[allow(unreachable_code)]
            Ok::<(), IpcError>(())
        }
    );
}

pub struct IPCServer {}
