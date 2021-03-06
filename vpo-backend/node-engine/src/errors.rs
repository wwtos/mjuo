use rhai::{EvalAltResult, ParseError};
use thiserror::Error;

use serde_json;

use crate::connection::SocketType;
use crate::node::NodeIndex;

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("Connection between {0} and {1} already exists")]
    AlreadyConnected(SocketType, SocketType),
    #[error("Input socket already occupied (Input {0})")]
    InputSocketOccupied(SocketType),
    #[error("Socket is not connected to any node")]
    NotConnected,
    #[error("Node does not exist in graph (index `{0}`)")]
    NodeDoesNotExist(NodeIndex),
    #[error("Mismatched node index: currently {0}, got {1}")]
    MismatchedNodeIndex(NodeIndex, NodeIndex),
    #[error("Node index `{0}` out of bounds")]
    IndexOutOfBounds(usize),
    #[error("Socket type `{0}` does not exist on node")]
    SocketDoesNotExist(SocketType),
    #[error("Socket types `{0}` and `{1}` are incompatible")]
    IncompatibleSocketTypes(SocketType, SocketType),
    #[error("Json parser error: `{0}`")]
    JsonParserError(#[from] serde_json::error::Error),
    #[error("Node type does not exist")]
    NodeTypeDoesNotExist,
    #[error("Property `{0}` missing!")]
    PropertyMissing(String),
    #[error("Socket by the name of `{0}` registered under different type")]
    RegistryCollision(String),
    #[error("Rhai parser error: {0}")]
    RhaiParserError(ParseError),
    #[error("Rhai evaluation error: {0}")]
    RhaiEvalError(EvalAltResult),
}

#[derive(Error, Debug)]
pub enum NodeWarning {
    #[error("Value of type `{0}` was returned, ignoring")]
    RhaiInvalidReturnType(String),
}

#[derive(Debug, Default)]
pub struct ErrorsAndWarnings {
    pub errors: Vec<NodeError>,
    pub warnings: Vec<NodeWarning>,
}

impl ErrorsAndWarnings {
    pub fn merge(mut self, other: Result<(), ErrorsAndWarnings>) -> Result<ErrorsAndWarnings, ErrorsAndWarnings> {
        if let Err(other) = other {
            if other.warnings.len() > 0 {
                self.warnings.extend(other.warnings);
            }

            if other.errors.len() > 0 {
                self.errors.extend(other.errors);
                Err(self)
            } else {
                Ok(self)
            }
        } else {
            Ok(self)
        }
    }
}
