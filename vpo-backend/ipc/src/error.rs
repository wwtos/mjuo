use snafu::Snafu;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum IpcError {
    #[snafu(display("Websocket error: {source}"))]
    #[cfg(any(unix, windows))]
    WebsocketError {
        source: tokio_tungstenite::tungstenite::Error,
    },
    #[snafu(display("Receive broadcast error: {source}"))]
    #[cfg(any(unix, windows))]
    ReceiveError {
        source: tokio::sync::broadcast::error::RecvError,
    },
    #[snafu(display("Dropped connection"))]
    DroppedConnection,
}
