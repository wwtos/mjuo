use rhai::{EvalAltResult, ParseError};
use thiserror::Error;

use serde_json;

use crate::connection::SocketType;
use crate::graph_manager::GraphIndex;
use crate::node::NodeIndex;

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("The field `{0}` was missing during an action rollback")]
    ActionRollbackFieldMissing(String),
    #[error("Graph does not exist at index `{0}`")]
    GraphDoesNotExist(GraphIndex),
    #[error("Graph has more than one parent, cannot remove")]
    GraphHasOtherParents,
    #[error("Connection between {0} and {1} already exists")]
    AlreadyConnected(SocketType, SocketType),
    #[error("Input socket already occupied (Input {0})")]
    InputSocketOccupied(SocketType),
    #[error("Socket is not connected to any node")]
    NotConnected,
    #[error("Node does not exist in graph (index `{0}`)")]
    NodeDoesNotExist(NodeIndex),
    #[error("Node already exists at index `{0}`")]
    NodeAlreadyExists(NodeIndex),
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
    #[error("Json parser error: `{0}` ({1})")]
    JsonParserErrorInContext(serde_json::error::Error, String),
    #[error("Node type does not exist")]
    NodeTypeDoesNotExist,
    #[error("Property `{0}` missing or malformed")]
    PropertyMissingOrMalformed(String),
    #[error("Socket by the name of `{0}` registered under different type")]
    RegistryCollision(String),
    #[error("Rhai parser error: {0}")]
    RhaiParserError(ParseError),
    #[error("Rhai evaluation error: {0}")]
    RhaiEvalError(EvalAltResult),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
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
    pub fn err(err: NodeError) -> ErrorsAndWarnings {
        ErrorsAndWarnings {
            errors: vec![err],
            warnings: vec![],
        }
    }

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
