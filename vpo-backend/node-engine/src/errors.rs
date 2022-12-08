use resource_manager::{LoadingError, ResourceId};
use rhai::{EvalAltResult, ParseError};
use semver::Version;
use snafu::Snafu;

use serde_json;

use crate::connection::{SocketDirection, SocketType};
use crate::graph_manager::GraphIndex;
use crate::node::{NodeIndex, NodeRow};

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum NodeError {
    #[snafu(display("The field `{missing_field}` was missing during an action rollback"))]
    ActionRollbackFieldMissing { missing_field: String },
    #[snafu(display("Graph does not exist at index `{graph_index}`"))]
    GraphDoesNotExist { graph_index: GraphIndex },
    #[snafu(display("Graph has more than one parent, cannot remove"))]
    GraphHasOtherParents,
    #[snafu(display("Connection between {from} and {to} already exists"))]
    AlreadyConnected { from: SocketType, to: SocketType },
    #[snafu(display("Input socket already occupied (Input {socket_type})"))]
    InputSocketOccupied { socket_type: SocketType },
    #[snafu(display("Socket is not connected to any node"))]
    NotConnected,
    #[snafu(display("Node does not exist in graph (index `{node_index}`)"))]
    NodeDoesNotExist { node_index: NodeIndex },
    #[snafu(display("Node already exists at index `{node_index}`"))]
    NodeAlreadyExists { node_index: NodeIndex },
    #[snafu(display("Mismatched node index: currently {current}, got {incoming}"))]
    MismatchedNodeIndex { current: NodeIndex, incoming: NodeIndex },
    #[snafu(display("Node index `{index}` out of bounds"))]
    IndexOutOfBounds { index: usize },
    #[snafu(display("Socket type `{socket_type}` does not exist on node"))]
    SocketDoesNotExist { socket_type: SocketType },
    #[snafu(display("Socket types `{from}` and `{to}` are incompatible"))]
    IncompatibleSocketTypes { from: SocketType, to: SocketType },
    #[snafu(display("Json parser error: `{source}`"))]
    JsonParserError { source: serde_json::error::Error },
    #[snafu(display("Json parser error: `{source}` ({context})"))]
    JsonParserErrorInContext {
        source: serde_json::error::Error,
        context: String,
    },
    #[snafu(display("Node type does not exist"))]
    NodeTypeDoesNotExist,
    #[snafu(display("Property `{property_name}` missing or malformed"))]
    PropertyMissingOrMalformed { property_name: String },
    #[snafu(display("Socket by the name of `{register_string}` registered under different type"))]
    RegistryCollision { register_string: String },
    #[snafu(display("Rhai evaluation error: {result}"))]
    RhaiEvalError { result: EvalAltResult },
    #[snafu(display("IO Error: {source}"))]
    IOError { source: std::io::Error },
    #[snafu(display("Inner graph errors: {errors_and_warnings:?}"))]
    InnerGraphErrors { errors_and_warnings: ErrorsAndWarnings },
    #[snafu(display("Missing resource: {resource:?}"))]
    MissingResource { resource: ResourceId },
    #[snafu(display("Loading error: {source:?}"))]
    LoadingError { source: LoadingError },
    #[snafu(display("Version doesn't exist: {version}"))]
    VersionError { version: Version },
    #[snafu(display("Trouble initializing midi: {source:?}"))]
    MidiInitError { source: midir::InitError },
}

impl NodeError {
    pub fn in_context(self, context: NodeErrorContext) -> ErrorInContext {
        ErrorInContext { error: self, context }
    }
}

impl From<ErrorInContext> for NodeError {
    fn from(error: ErrorInContext) -> Self {
        error.error
    }
}

#[derive(Snafu, Debug)]
pub enum NodeWarning {
    #[snafu(display("Value of type `{return_type}` was returned, ignoring"))]
    RhaiInvalidReturnType { return_type: String },
    #[snafu(display("Rhai execution failed: {err} (script: `{script}`)"))]
    RhaiExecutionFailure { err: EvalAltResult, script: String },
    #[snafu(display("Rhai parser failure: {parser_error}"))]
    RhaiParserFailure { parser_error: ParseError },
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

impl Warnings {
    pub fn warning(warning: NodeWarning) -> Warnings {
        Warnings {
            warnings: vec![warning],
        }
    }
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
            if !other.warnings.is_empty() {
                self.warnings.extend(other.warnings);
            }

            if !other.errors.is_empty() {
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

impl Default for WarningBuilder {
    fn default() -> Self {
        Self::new()
    }
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
