use ddgg::GraphError;
use resource_manager::ResourceId;
use rhai::{EvalAltResult, ParseError};
use snafu::Snafu;

use crate::connection::{Socket, SocketType};
use crate::graph_manager::{ConnectedThrough, GlobalNodeIndex, GraphIndex};
use crate::node::NodeIndex;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum NodeError {
    #[snafu(display("Node `{from:?}` and `{to:?}` are on two different graphs"))]
    MismatchedNodeGraphs { from: GlobalNodeIndex, to: GlobalNodeIndex },
    #[snafu(display("The field `{missing_field}` was missing during an action rollback"))]
    ActionRollbackFieldMissing { missing_field: String },
    #[snafu(display("Graph does not exist at index `{graph_index:?}`"))]
    GraphDoesNotExist { graph_index: GraphIndex },
    #[snafu(display("Graph has more than one parent, cannot remove"))]
    GraphHasOtherParents,
    #[snafu(display("Graphs `{from:?}` and `{to:?}` not connected through `{through:?}`"))]
    GraphsNotConnected {
        from: GraphIndex,
        through: ConnectedThrough,
        to: GraphIndex,
    },
    #[snafu(display(
        "Connection does not exist between {from_index:?} {from_socket:?} and {to_index:?} {to_socket:?}"
    ))]
    NodesNotConnected {
        from_index: NodeIndex,
        from_socket: Socket,
        to_index: NodeIndex,
        to_socket: Socket,
    },
    #[snafu(display("Connection between {from:?} and {to:?} already exists"))]
    AlreadyConnected { from: Socket, to: Socket },
    #[snafu(display("Input socket already occupied (Input {socket:?})"))]
    InputSocketOccupied { socket: Socket },
    #[snafu(display("Node does not exist in graph (index `{node_index:?}`)"))]
    NodeDoesNotExist { node_index: NodeIndex },
    #[snafu(display("Node already exists at index `{node_index:?}`"))]
    NodeAlreadyExists { node_index: NodeIndex },
    #[snafu(display("Mismatched node index: currently {current:?}, got {incoming:?}"))]
    MismatchedNodeIndex { current: NodeIndex, incoming: NodeIndex },
    #[snafu(display("Node index `{index}` out of bounds"))]
    IndexOutOfBounds { index: usize },
    #[snafu(display("Socket type `{socket:?}` does not exist on node"))]
    SocketDoesNotExist { socket: Socket },
    #[snafu(display("Socket types `{from:?}` and `{to:?}` are incompatible"))]
    IncompatibleSocketTypes { from: SocketType, to: SocketType },
    #[snafu(display("Node type does not exist"))]
    NodeTypeDoesNotExist,
    #[snafu(display("Rhai evaluation error: {result}"))]
    RhaiEvalError { result: EvalAltResult },
    #[snafu(display("Inner graph errors: {errors_and_warnings:?}"))]
    InnerGraphErrors { errors_and_warnings: ErrorsAndWarnings },
    #[snafu(display("Missing resource: {resource:?}"))]
    MissingResource { resource: ResourceId },
    #[snafu(display("Graph error: {error:?}"))]
    GraphError { error: GraphError },
    #[snafu(display("Expected node type {expected}, got {actual}"))]
    IncorrectNodeType { expected: String, actual: String },
    #[snafu(display("Can't delete root node"))]
    CannotDeleteRootNode,
}

impl From<GraphError> for NodeError {
    fn from(value: GraphError) -> Self {
        NodeError::GraphError { error: value }
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
    #[snafu(display("Internal node errors/warnings: {errors_and_warnings:?}"))]
    InternalErrorsAndWarnings { errors_and_warnings: ErrorsAndWarnings },
}

pub type NodeResult<T> = Result<NodeOk<T>, NodeError>;

#[derive(Debug, Default)]
pub struct ErrorsAndWarnings {
    pub errors: Vec<(NodeIndex, NodeError)>,
    pub warnings: Vec<(NodeIndex, NodeWarning)>,
}

impl ErrorsAndWarnings {
    pub fn any(&self) -> bool {
        !self.errors.is_empty() || !self.warnings.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct NodeOk<T> {
    pub value: T,
    pub warnings: Vec<NodeWarning>,
}

impl<T> NodeOk<T> {
    pub fn no_warnings(value: T) -> NodeResult<T> {
        Ok(NodeOk::<T> {
            value,
            warnings: vec![],
        })
    }

    pub fn new(value: T, warnings: Vec<NodeWarning>) -> NodeOk<T> {
        NodeOk { value, warnings }
    }
}

pub trait WarningExt<T> {
    fn append_warnings(self, warning_builder: &mut Vec<NodeWarning>) -> Result<T, NodeError>;
}

impl<T> WarningExt<T> for Result<NodeOk<T>, NodeError> {
    fn append_warnings(self, warnings: &mut Vec<NodeWarning>) -> Result<T, NodeError> {
        self.map(|mut node_ok| {
            warnings.append(&mut node_ok.warnings);

            node_ok.value
        })
    }
}
