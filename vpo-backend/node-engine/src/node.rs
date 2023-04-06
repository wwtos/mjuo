//! Node module

use std::collections::HashMap;
use std::fmt::{Debug, Display};

use ddgg::VertexIndex;
use enum_dispatch::enum_dispatch;
use rhai::Engine;
use serde::{Deserialize, Serialize};

use crate::connection::{MidiBundle, Primitive, Socket, SocketDirection, SocketType, SocketValue};

use crate::errors::{NodeOk, NodeResult};
use crate::global_state::GlobalState;
use crate::graph_manager::{GraphIndex, GraphManager};
use crate::property::{Property, PropertyType};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "variant", content = "data")]
pub enum NodeRow {
    // type, value, polyphonic?
    Input(Socket, SocketValue),
    Output(Socket, SocketValue),
    Property(String, PropertyType, Property),
    InnerGraph,
}

impl NodeRow {
    pub fn to_socket_and_direction(&self) -> Option<(Socket, SocketDirection)> {
        match self {
            NodeRow::Input(socket, _) => Some((*socket, SocketDirection::Input)),
            NodeRow::Output(socket, _) => Some((*socket, SocketDirection::Output)),
            NodeRow::Property(..) => None,
            NodeRow::InnerGraph => None,
        }
    }

    pub fn from_type_and_direction(socket: Socket, direction: SocketDirection) -> Self {
        match direction {
            SocketDirection::Input => NodeRow::Input(socket, SocketValue::None),
            SocketDirection::Output => NodeRow::Output(socket, SocketValue::None),
        }
    }
}

pub fn stream_input(uid: u32, default: f32) -> NodeRow {
    NodeRow::Input(Socket::Simple(uid, SocketType::Stream, 1), SocketValue::Stream(default))
}

pub fn midi_input(uid: u32, default: MidiBundle) -> NodeRow {
    NodeRow::Input(Socket::Simple(uid, SocketType::Midi, 1), SocketValue::Midi(default))
}

pub fn value_input(uid: u32, default: Primitive) -> NodeRow {
    NodeRow::Input(Socket::Simple(uid, SocketType::Value, 1), SocketValue::Value(default))
}

pub fn stream_output(uid: u32, default: f32) -> NodeRow {
    NodeRow::Output(Socket::Simple(uid, SocketType::Stream, 1), SocketValue::Stream(default))
}

pub fn midi_output(uid: u32, default: MidiBundle) -> NodeRow {
    NodeRow::Output(Socket::Simple(uid, SocketType::Midi, 1), SocketValue::Midi(default))
}

pub fn value_output(uid: u32, default: Primitive) -> NodeRow {
    NodeRow::Output(Socket::Simple(uid, SocketType::Value, 1), SocketValue::Value(default))
}

pub struct NodeIo {
    pub node_rows: Vec<NodeRow>,
    pub child_graph_io: Option<Vec<(Socket, SocketDirection)>>,
}

impl NodeIo {
    pub fn simple(node_rows: Vec<NodeRow>) -> NodeIo {
        NodeIo {
            node_rows,
            child_graph_io: None,
        }
    }
}

pub struct InitResult {
    pub changed_properties: Option<HashMap<String, Property>>,
}

impl InitResult {
    pub fn nothing() -> NodeResult<InitResult> {
        NodeOk::no_warnings(InitResult {
            changed_properties: None,
        })
    }
}

pub struct NodeInitState<'a> {
    pub props: &'a HashMap<String, Property>,
    pub script_engine: &'a Engine,
    pub global_state: &'a GlobalState,
    pub graph_manager: &'a GraphManager,
    pub current_time: i64,
}

pub struct NodeProcessState<'a> {
    pub current_time: i64,
    pub script_engine: &'a Engine,
    pub global_state: &'a GlobalState,
}

pub struct NodeGraphAndIo {
    pub graph: GraphIndex,
    pub input_index: NodeIndex,
    pub output_index: NodeIndex,
}

/// NodeRuntime trait
///
/// This is the most fundamental building block of a graph node network.
/// It is the part of the graph that does the actual thinking. Data is presented to it
/// through its sockets. The graph will call `list_input_sockets` and `list_output_sockets`
/// to determine what sockets the node has available. From then, the graph will take care
/// of data flow, connecting nodes together, and such.
///
///  It needs to implement methods listing
/// what properties it has, what sockets it has available to
#[allow(unused_variables)]
#[enum_dispatch(NodeVariant)]
pub trait NodeRuntime: Debug + Clone {
    fn init(&mut self, state: NodeInitState, child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        InitResult::nothing()
    }

    fn linked_to_ui(&self) -> bool {
        false
    }

    /// Process received data.
    fn process(&mut self, state: NodeProcessState, streams_in: &[f32], streams_out: &mut [f32]) -> NodeResult<()> {
        NodeOk::no_warnings(())
    }

    /// Accept incoming midi data (ordered based on rows returned from `init`)
    fn accept_midi_inputs(&mut self, midi_in: &[Option<MidiBundle>]) {}

    /// Return outgoing midi data (ordered based on rows returned from `init`)
    fn get_midi_outputs(&self, midi_out: &mut [Option<MidiBundle>]) {}

    /// Accept incoming value data (ordered based on rows returned from `init`)
    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {}

    /// Return outgoing value data (ordered based on rows returned from `init`)
    fn get_value_outputs(&self, values_out: &mut [Option<Primitive>]) {}
}

/// A static method returning a node's IO list. Note this is dynamic, but it cannot be dependent on
/// internal state
pub trait Node: NodeRuntime {
    /// Called at least every time a property is changed
    fn get_io(props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct NodeIndex(pub VertexIndex);

impl Display for NodeIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.0)
    }
}
