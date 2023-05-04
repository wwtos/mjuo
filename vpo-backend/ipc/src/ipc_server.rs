use futures::{try_join, SinkExt, StreamExt};
use log::info;
use snafu::ResultExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::{self, Message};

use crate::error::{IpcError, ReceiveSnafu, WebsocketSnafu};
use crate::ipc_message::IpcMessage;

pub async fn start_ipc(to_tokio: broadcast::Sender<IpcMessage>, to_main: broadcast::Sender<IpcMessage>) {
    let listener = TcpListener::bind("127.0.0.1:26642").await.expect("failed to bind");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(create_connection_task(stream, to_tokio.subscribe(), to_main.clone()));
    }
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
                let msg = from_main.recv().await.context(ReceiveSnafu)?;

                let IpcMessage::Json(json) = msg;

                let err = to_client
                    .send(Message::Text(serde_json::to_string(&json).unwrap()))
                    .await;

                if let Err(tungstenite::error::Error::ConnectionClosed) = err {
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
