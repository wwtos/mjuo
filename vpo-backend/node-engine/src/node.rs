//! Node module

use std::collections::HashMap;
use std::fmt::{Debug, Display};

use ddgg::VertexIndex;
use enum_dispatch::enum_dispatch;
use rhai::Engine;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::connection::{
    MidiBundle, MidiSocketType, NodeRefSocketType, Primitive, SocketDirection, SocketType, StreamSocketType,
    ValueSocketType,
};

use crate::errors::{NodeError, NodeOk, NodeResult};
use crate::global_state::GlobalState;
use crate::node_graph::NodeGraph;
use crate::property::{Property, PropertyType};
use crate::socket_registry::SocketRegistry;
use crate::traversal::traverser::Traverser;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "variant", content = "data")]
pub enum NodeRow {
    // type, value, polyphonic?
    StreamInput(StreamSocketType, f32, bool),
    MidiInput(MidiSocketType, MidiBundle, bool),
    ValueInput(ValueSocketType, Primitive, bool),
    NodeRefInput(NodeRefSocketType, bool),
    StreamOutput(StreamSocketType, f32, bool),
    MidiOutput(MidiSocketType, MidiBundle, bool),
    ValueOutput(ValueSocketType, Primitive, bool),
    NodeRefOutput(NodeRefSocketType, bool),
    Property(String, PropertyType, Property),
    InnerGraph,
}

impl NodeRow {
    pub fn to_type_and_direction(self) -> Option<(SocketType, SocketDirection)> {
        match self {
            NodeRow::StreamInput(stream_type, ..) => Some((SocketType::Stream(stream_type), SocketDirection::Input)),
            NodeRow::MidiInput(midi_type, ..) => Some((SocketType::Midi(midi_type), SocketDirection::Input)),
            NodeRow::ValueInput(value_type, ..) => Some((SocketType::Value(value_type), SocketDirection::Input)),
            NodeRow::NodeRefInput(node_ref_type, ..) => {
                Some((SocketType::NodeRef(node_ref_type), SocketDirection::Input))
            }
            NodeRow::StreamOutput(stream_type, ..) => Some((SocketType::Stream(stream_type), SocketDirection::Output)),
            NodeRow::MidiOutput(midi_type, ..) => Some((SocketType::Midi(midi_type), SocketDirection::Output)),
            NodeRow::ValueOutput(value_type, ..) => Some((SocketType::Value(value_type), SocketDirection::Output)),
            NodeRow::NodeRefOutput(node_ref_type, ..) => {
                Some((SocketType::NodeRef(node_ref_type), SocketDirection::Output))
            }
            NodeRow::Property(..) => None,
            NodeRow::InnerGraph => None,
        }
    }

    pub fn from_type_and_direction(socket_type: SocketType, direction: SocketDirection, polyphonic: bool) -> Self {
        match direction {
            SocketDirection::Input => match socket_type {
                SocketType::Stream(stream_type) => NodeRow::StreamInput(stream_type, 0.0, polyphonic),
                SocketType::Midi(midi_type) => NodeRow::MidiInput(midi_type, SmallVec::new(), polyphonic),
                SocketType::Value(value_type) => NodeRow::ValueInput(value_type, Primitive::Float(0.0), polyphonic),
                SocketType::NodeRef(node_ref_type) => NodeRow::NodeRefInput(node_ref_type, polyphonic),
            },
            SocketDirection::Output => match socket_type {
                SocketType::Stream(stream_type) => NodeRow::StreamOutput(stream_type, 0.0, polyphonic),
                SocketType::Midi(midi_type) => NodeRow::MidiOutput(midi_type, SmallVec::new(), polyphonic),
                SocketType::Value(value_type) => NodeRow::ValueOutput(value_type, Primitive::Float(0.0), polyphonic),
                SocketType::NodeRef(node_ref_type) => NodeRow::NodeRefOutput(node_ref_type, polyphonic),
            },
        }
    }
}

pub struct InitResult {
    pub did_rows_change: bool,
    pub node_rows: Vec<NodeRow>,
    pub changed_properties: Option<HashMap<String, Property>>,
}

impl InitResult {
    pub fn simple(node_rows: Vec<NodeRow>) -> NodeResult<InitResult> {
        NodeOk::no_warnings(InitResult {
            did_rows_change: false,
            node_rows,
            changed_properties: None,
        })
    }
}

pub struct NodeInitState<'a> {
    pub props: &'a HashMap<String, Property>,
    pub registry: &'a mut SocketRegistry,
    pub script_engine: &'a Engine,
    pub global_state: &'a GlobalState,
}

pub struct NodeProcessState<'a> {
    pub current_time: i64,
    pub script_engine: &'a Engine,
    pub child_graph: Option<(&'a mut NodeGraph, &'a Traverser)>,
    pub global_state: &'a GlobalState,
}

/// Node trait
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
pub trait Node: Debug + Clone {
    fn init(&mut self, state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError>;

    fn get_child_graph_socket_list(&self, registry: &mut SocketRegistry) -> Vec<(SocketType, SocketDirection)> {
        vec![]
    }

    fn init_graph(&mut self, graph: &mut NodeGraph, input_node: NodeIndex, output_node: NodeIndex) {}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct NodeIndex(pub VertexIndex);

impl Display for NodeIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.0)
    }
}
