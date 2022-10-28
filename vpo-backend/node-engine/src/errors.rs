use rhai::{EvalAltResult, ParseError};
use thiserror::Error;

use serde_json;

use crate::connection::{SocketDirection, SocketType};
use crate::graph_manager::GraphIndex;
use crate::node::{NodeIndex, NodeRow};

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

impl NodeError {
    pub fn in_context(self, context: NodeErrorContext) -> ErrorInContext {
        ErrorInContext {
            error: self,
            context: context,
        }
    }
}

impl From<ErrorInContext> for NodeError {
    fn from(error: ErrorInContext) -> Self {
        error.error
    }
}

#[derive(Error, Debug)]
pub enum NodeWarning {
    #[error("Value of type `{0}` was returned, ignoring")]
    RhaiInvalidReturnType(String),
}

impl From<WarningInContext> for NodeWarning {
    fn from(warning: WarningInContext) -> Self {
        warning.warning
    }
}

#[derive(Debug)]
pub struct NodeErrorContext {
    pub graph: Option<GraphIndex>,
    pub index: Option<NodeIndex>,
    pub socket: Option<SocketType>,
    pub direction: Option<SocketDirection>,
    pub node_row: Option<NodeRow>,
}

pub struct ErrorInContext {
    pub error: NodeError,
    pub context: NodeErrorContext,
}

pub struct WarningInContext {
    pub warning: NodeWarning,
    pub context: NodeErrorContext,
}

pub type NodeResult<T> = Result<NodeOk<T>, NodeError>;

#[derive(Debug)]
pub struct Warnings {
    pub warnings: Vec<NodeWarning>,
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

    pub fn merge(self, other: Result<(), ErrorsAndWarnings>) -> Result<ErrorsAndWarnings, ErrorsAndWarnings> {
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

#[derive(Debug, Default)]
pub struct NodeOk<T> {
    pub value: T,
    pub warnings: Option<Warnings>,
}

impl<T> NodeOk<T> {
    pub fn no_warnings(value: T) -> NodeResult<T> {
        Ok(NodeOk::<T> { value, warnings: None })
    }

    pub fn new(value: T, warnings: Option<Warnings>) -> NodeOk<T> {
        NodeOk { value, warnings }
    }
}

pub struct WarningBuilder {
    warnings: Option<Vec<NodeWarning>>,
}

impl WarningBuilder {
    pub fn new() -> WarningBuilder {
        WarningBuilder { warnings: None }
    }

    fn warnings_ref(&mut self) -> &mut Vec<NodeWarning> {
        if self.warnings.is_none() {
            self.warnings = Some(Vec::new());
        }

        self.warnings.as_mut().unwrap()
    }

    pub fn append_warnings(&mut self, warnings: Option<Warnings>) {
        if let Some(warnings) = warnings {
            self.warnings_ref().extend(warnings.warnings);
        }
    }

    pub fn add_warning(&mut self, warning: NodeWarning) {
        self.warnings_ref().push(warning);
    }

    pub fn into_warnings(self) -> Option<Warnings> {
        self.warnings.map(|warnings| Warnings { warnings })
    }
}
