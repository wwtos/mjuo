use snafu::Snafu;
use tokio_tungstenite::tungstenite;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum IpcError {
    #[snafu(display("Websocket error: {source}"))]
    #[cfg(any(unix, windows))]
    WebsocketError { source: tungstenite::Error },
    #[snafu(display("Dropped connection"))]
    DroppedConnection,
}
