//! Node module

use std::any::Any;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Display};

use ddgg::VertexIndex;
use enum_dispatch::enum_dispatch;
use resource_manager::ResourceId;
use rhai::Engine;
use serde::{Deserialize, Serialize};
use sound_engine::SoundConfig;

use crate::connection::{MidiBundle, Primitive, Socket, SocketDirection, SocketValue};

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
    pub fn to_socket_and_direction(&self) -> Option<(&Socket, SocketDirection)> {
        match self {
            NodeRow::Input(socket, _) => Some((socket, SocketDirection::Input)),
            NodeRow::Output(socket) => Some((socket, SocketDirection::Output)),
            NodeRow::Property(..) => None,
            NodeRow::InnerGraph => None,
        }
    }

    pub fn to_socket_and_value(&self) -> Option<(&Socket, SocketValue)> {
        match self {
            NodeRow::Input(socket, value) => Some((socket, value.clone())),
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
    pub needed_resources: Vec<ResourceId>,
}

impl InitResult {
    pub fn nothing() -> NodeResult<InitResult> {
        NodeOk::no_warnings(InitResult {
            changed_properties: None,
            needed_resources: vec![],
        })
    }

    pub fn warning(warning: Option<NodeWarning>) -> NodeResult<InitResult> {
        Ok(NodeOk::new(
            InitResult {
                changed_properties: None,
                needed_resources: vec![],
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

pub struct NodeInitParams<'a> {
    pub props: &'a HashMap<String, Property>,
    pub script_engine: &'a Engine,
    pub resources: &'a Resources,
    pub graph_manager: &'a GraphManager,
    pub current_time: i64,
    pub sound_config: &'a SoundConfig,
    pub node_state: &'a NodeState,
    pub child_graph: Option<NodeGraphAndIo>,
}

pub struct StateInterface<'a> {
    pub request_node_states: &'a mut dyn FnMut(),
    pub enqueue_state_updates: &'a mut dyn FnMut(Vec<(NodeIndex, serde_json::Value)>),
    pub states: Option<&'a BTreeMap<NodeIndex, NodeState>>,
}

pub struct NodeProcessContext<'a> {
    pub current_time: i64,
    pub resources: &'a Resources,
    pub script_engine: &'a Engine,
    pub external_state: StateInterface<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGraphAndIo {
    pub graph_index: GraphIndex,
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

pub struct Ins<'a> {
    pub midis: &'a [&'a [MidiBundle]],
    pub values: &'a [&'a [Primitive]],
    pub streams: &'a [&'a [&'a [f32]]],
}

pub struct Outs<'a> {
    pub midis: &'a mut [&'a mut [MidiBundle]],
    pub values: &'a mut [&'a mut [Primitive]],
    pub streams: &'a mut [&'a mut [&'a mut [f32]]],
}

/// NodeRuntime trait
///
/// This is the most fundamental building block of a graph node network.
/// It is the part of the graph that does the actual thinking. Data is presented to it
/// through its sockets, which are returned by implementing the `Node` trait.
///
/// ## Life cycle
/// `init` is called on first creation, and any time a property changes.
///
/// `has_state` is called when it is first created. It returns whether the state has a
/// state that needs to be tracked.
///
/// ### Runtime
/// `set_state`, `accept_midi_inputs`, and `accept_value_inputs` are called before `process`
/// in an arbitrary order. They are also _only_ called if there is new incoming state.
///
/// `process` is called after that, and this is where most of the work happens.
///
/// After that, `get_midi_outputs`, `get_value_outputs` are called every time, and
/// that's how values are returned out. `get_state` is also called at this time
/// if the node has state to track.
///
/// To wrap up, `finish` is called at the end of processing.
#[allow(unused_variables)]
#[enum_dispatch(NodeVariant)]
pub trait NodeRuntime: Debug + Clone {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        InitResult::nothing()
    }

    /// Called once to check if it's stateful
    fn has_state(&self) -> bool {
        false
    }

    fn get_state(&self) -> Option<NodeState> {
        None
    }

    fn set_state(&mut self, state: serde_json::Value) {}

    /// Process all data in and out
    fn process(&mut self, context: NodeProcessContext, ins: Ins, outs: Outs, resources: &[&dyn Any]) -> NodeResult<()> {
        ProcessResult::nothing()
    }
}

/// A static method returning a node's IO list. Note this is dynamic, but it cannot be dependent on
/// internal state
pub trait Node: NodeRuntime {
    /// Called at least every time a property is changed
    fn get_io(props: HashMap<String, Property>) -> NodeIo;

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
