//! Node module

use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Display};

use ddgg::VertexIndex;
use enum_dispatch::enum_dispatch;
use rhai::Engine;
use serde::{Deserialize, Serialize};
use sound_engine::SoundConfig;

use crate::connection::{MidiBundle, Primitive, Socket, SocketDirection, SocketType, SocketValue};

use crate::errors::{NodeOk, NodeResult, NodeWarning};
use crate::global_state::Resources;
use crate::graph_manager::{GraphIndex, GraphManager};
use crate::property::{Property, PropertyType};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "variant", content = "data")]
pub enum NodeRow {
    // type, value, polyphonic?
    Input(Socket, SocketValue),
    Output(Socket),
    Property(String, PropertyType, Property),
    InnerGraph,
}

impl NodeRow {
    pub fn to_socket_and_direction(&self) -> Option<(Socket, SocketDirection)> {
        match self {
            NodeRow::Input(socket, _) => Some((*socket, SocketDirection::Input)),
            NodeRow::Output(socket) => Some((*socket, SocketDirection::Output)),
            NodeRow::Property(..) => None,
            NodeRow::InnerGraph => None,
        }
    }

    pub fn to_socket_and_value(&self) -> Option<(Socket, SocketValue)> {
        match self {
            NodeRow::Input(socket, value) => Some((*socket, value.clone())),
            NodeRow::Output(_) => None,
            NodeRow::Property(..) => None,
            NodeRow::InnerGraph => None,
        }
    }

    pub fn from_type_and_direction(socket: Socket, direction: SocketDirection) -> Self {
        match direction {
            SocketDirection::Input => NodeRow::Input(socket, SocketValue::None),
            SocketDirection::Output => NodeRow::Output(socket),
        }
    }
}

pub fn stream_input(uid: u32) -> NodeRow {
    NodeRow::Input(Socket::Simple(uid, SocketType::Stream, 1), SocketValue::None)
}

pub fn midi_input(uid: u32) -> NodeRow {
    NodeRow::Input(Socket::Simple(uid, SocketType::Midi, 1), SocketValue::None)
}

pub fn value_input(uid: u32, default: Primitive) -> NodeRow {
    NodeRow::Input(Socket::Simple(uid, SocketType::Value, 1), SocketValue::Value(default))
}

pub fn stream_output(uid: u32) -> NodeRow {
    NodeRow::Output(Socket::Simple(uid, SocketType::Stream, 1))
}

pub fn midi_output(uid: u32) -> NodeRow {
    NodeRow::Output(Socket::Simple(uid, SocketType::Midi, 1))
}

pub fn value_output(uid: u32) -> NodeRow {
    NodeRow::Output(Socket::Simple(uid, SocketType::Value, 1))
}

pub fn property(prop_id: &str, prop_type: PropertyType, prop_default: Property) -> NodeRow {
    NodeRow::Property(prop_id.to_string(), prop_type, prop_default)
}

pub fn multiple_choice(prop_id: &str, choices: &[&str], default_choice: &str) -> NodeRow {
    NodeRow::Property(
        prop_id.to_string(),
        PropertyType::MultipleChoice(choices.iter().map(|&choice| choice.to_string()).collect()),
        Property::MultipleChoice(default_choice.to_string()),
    )
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

    pub fn warning(warning: Option<NodeWarning>) -> NodeResult<InitResult> {
        Ok(NodeOk::new(
            InitResult {
                changed_properties: None,
            },
            warning.map(|x| vec![x]).unwrap_or(vec![]),
        ))
    }
}

pub struct ProcessResult {}

impl ProcessResult {
    pub fn nothing() -> NodeResult<()> {
        NodeOk::no_warnings(())
    }

    pub fn warning(warning: Option<NodeWarning>) -> NodeResult<()> {
        Ok(NodeOk::new((), warning.map(|x| vec![x]).unwrap_or(vec![])))
    }
}

pub struct NodeInitState<'a> {
    pub props: &'a HashMap<String, Property>,
    pub script_engine: &'a Engine,
    pub resources: &'a Resources,
    pub graph_manager: &'a GraphManager,
    pub current_time: i64,
    pub sound_config: &'a SoundConfig,
    pub state: &'a NodeState,
}

pub struct StateInterface<'a> {
    pub request_node_states: &'a mut dyn FnMut(),
    pub enqueue_state_updates: &'a mut dyn FnMut(Vec<(NodeIndex, serde_json::Value)>),
    pub states: Option<&'a BTreeMap<NodeIndex, NodeState>>,
}

pub struct NodeProcessState<'a> {
    pub current_time: i64,
    pub resources: &'a Resources,
    pub script_engine: &'a Engine,
    pub state: StateInterface<'a>,
}

pub struct NodeGraphAndIo {
    pub graph: GraphIndex,
    pub input_index: NodeIndex,
    pub output_index: NodeIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeState {
    pub counted_during_mapset: bool,
    pub value: serde_json::Value,
    pub other: serde_json::Value,
}

impl Default for NodeState {
    fn default() -> Self {
        NodeState {
            counted_during_mapset: false,
            value: serde_json::Value::Null,
            other: serde_json::Value::Null,
        }
    }
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

    fn has_state(&self) -> bool {
        false
    }

    fn get_state(&self) -> Option<NodeState> {
        None
    }

    fn set_state(&mut self, state: serde_json::Value) {}

    /// Process received data.
    fn process(
        &mut self,
        state: NodeProcessState,
        streams_in: &[&[f32]],
        streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        NodeOk::no_warnings(())
    }

    /// Accept incoming midi data (ordered based on rows returned from `init`)
    fn accept_midi_inputs(&mut self, midi_in: &[Option<MidiBundle>]) {}

    /// Return outgoing midi data (ordered based on rows returned from `init`)
    fn get_midi_outputs(&mut self, midi_out: &mut [Option<MidiBundle>]) {}

    /// Accept incoming value data (ordered based on rows returned from `init`)
    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {}

    /// Return outgoing value data (ordered based on rows returned from `init`)
    fn get_value_outputs(&mut self, values_out: &mut [Option<Primitive>]) {}

    /// Runs at the end every frame
    fn finish(&mut self) {}
}

/// A static method returning a node's IO list. Note this is dynamic, but it cannot be dependent on
/// internal state
pub trait Node: NodeRuntime {
    /// Called at least every time a property is changed
    fn get_io(props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo;

    /// Called when created
    fn new(sound_config: &SoundConfig) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct NodeIndex(pub VertexIndex);

impl Display for NodeIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.0)
    }
}
