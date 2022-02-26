use thiserror::Error;

use serde_json;

use crate::{node::NodeIndex, connection::SocketType};

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("Connection between {0} and {1} already exists")]
    AlreadyConnected(SocketType, SocketType),
    #[error("Socket is not connected to any node")]
    NotConnected,
    #[error("Node does not exist in graph (index `{0}`)")]
    NodeDoesNotExist(NodeIndex),
    #[error("Node index `{0}` out of bounds")]
    IndexOutOfBounds(usize),
    #[error("Socket type `{0}` does not exist on node")]
    SocketDoesNotExist(SocketType),
    #[error("Socket types `{0}` and `{1}` are incompatible")]
    IncompatibleSocketTypes(SocketType, SocketType),
    #[error("Json parser error")]
    JsonParserError(#[from] serde_json::error::Error),
    #[error("Node type does not exist")]
    NodeTypeDoesNotExist
}
