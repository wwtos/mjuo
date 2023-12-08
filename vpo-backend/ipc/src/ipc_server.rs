use std::thread::{self, JoinHandle};

use tokio::net::{TcpListener, TcpStream};
use tokio::runtime;

use futures::{try_join, SinkExt, StreamExt};
use log::info;
use snafu::ResultExt;
use tokio::sync::broadcast;
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

            let (from_main_sender, _) = broadcast::channel(256);
            let (to_main_sender, _) = broadcast::channel(256);

            let sender_clone = from_main_sender.clone();
            tokio::spawn(async move {
                while let Ok(msg) = from_main.recv_async().await {
                    let _ = sender_clone.send(msg);
                }
            });

            let mut receiver = to_main_sender.subscribe();
            tokio::spawn(async move {
                while let Ok(msg) = receiver.recv().await {
                    let _ = to_main.send(msg);
                }
            });

            // split incoming messages

            while let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(create_connection_task(
                    stream,
                    from_main_sender.subscribe(),
                    to_main_sender.clone(),
                ));
            }
        });
    })
}

async fn create_connection_task(
    stream: TcpStream,
    mut from_main: broadcast::Receiver<IpcMessage>,
    to_main: broadcast::Sender<IpcMessage>,
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

    println!("opening connection");

    let receiving_block = async move {
        while let Some(msg) = from_client.next().await {
            let msg = msg.context(WebsocketSnafu)?;

            if let Message::Text(text) = msg {
                if let Ok(json) = serde_json::from_str(&text) {
                    to_main.send(IpcMessage::Json(json)).unwrap();
                }
            }
        }

        println!("closed connection");

        Ok::<(), IpcError>(())
    };

    let sending_block = async move {
        loop {
            let msg = from_main.recv().await.unwrap();
            dbg!(&msg);

            let IpcMessage::Json(json) = msg;

            let res = to_client
                .send(Message::Text(serde_json::to_string(&json).unwrap()))
                .await;

            match res {
                Ok(()) => {}
                Err(_) => break,
            }
        }

        // for type annotations
        #[allow(unreachable_code)]
        Ok::<(), IpcError>(())
    };

    let _ = try_join!(receiving_block, sending_block);
}

pub struct IPCServer {}
