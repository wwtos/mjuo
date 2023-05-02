use snafu::Snafu;

use crate::ipc_message::IpcMessage;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum IpcError {
    #[snafu(display("Websocket error: {source}"))]
    WebsocketError {
        source: tokio_tungstenite::tungstenite::Error,
    },
    #[snafu(display("Receive broadcast error: {source}"))]
    ReceiveError {
        source: tokio::sync::broadcast::error::RecvError,
    },
}
