//! Node module

pub mod buffered_traverser;
pub mod calculate_traversal_order;
pub mod osc_store;

use std::cell::UnsafeCell;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::mem;
use std::ops::{Index, IndexMut};
use std::time::Duration;

use common::resource_manager::ResourceId;
use common::SeaHashMap;
use ddgg::VertexIndex;
use enum_dispatch::enum_dispatch;
use rhai::Engine;
use serde::{Deserialize, Serialize};
use sound_engine::SoundConfig;

use crate::connection::{Primitive, Socket, SocketDirection, SocketValue};

use crate::errors::{NodeOk, NodeResult, NodeWarning};
use crate::graph_manager::{GraphIndex, GraphManager};
use crate::node_graph::NodeGraph;
use crate::property::{Property, PropertyType};
use crate::resources::{Resource, Resources};

use self::osc_store::OscStore;

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

#[derive(Debug)]
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

#[derive(Debug, Default)]
pub struct InitResult {
    pub changed_properties: Option<SeaHashMap<String, Property>>,
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

#[derive(Debug)]
pub struct NodeGetIoContext<'a> {
    pub default_channel_count: usize,
    pub connected_inputs: Vec<Socket>,
    pub connected_outputs: Vec<Socket>,
    pub child_graph: Option<&'a NodeGraph>,
}

impl<'a> NodeGetIoContext<'a> {
    pub fn no_io_yet(default_channel_count: usize) -> NodeGetIoContext<'a> {
        NodeGetIoContext {
            default_channel_count,
            connected_inputs: vec![],
            connected_outputs: vec![],
            child_graph: None,
        }
    }
}

pub struct NodeInitParams<'a> {
    pub props: &'a SeaHashMap<String, Property>,
    pub script_engine: &'a Engine,
    pub resources: &'a Resources,
    pub graph_manager: &'a GraphManager,
    pub current_time: Duration,
    pub sound_config: SoundConfig,
    pub node_state: &'a NodeState,
    pub child_graph: Option<GraphIndex>,
    pub default_channel_count: usize,
}

impl<'a> NodeInitParams<'a> {
    /// This gets the channel count based on a property called `channels` that should be provided.
    /// If not, it defaults to the global default channel count
    pub fn get_channel_count(&self) -> usize {
        match self.props.get("channels") {
            Some(prop) => prop
                .as_integer()
                .map(|x| x.max(1) as usize)
                .unwrap_or(self.default_channel_count),
            None => self.default_channel_count,
        }
    }
}

pub struct StateInterface<'a> {
    pub request_node_states: &'a mut dyn FnMut(),
    pub enqueue_state_updates: &'a mut dyn FnMut(Vec<(NodeIndex, serde_json::Value)>),
    pub states: Option<&'a BTreeMap<NodeIndex, NodeState>>,
}

pub struct NodeProcessContext<'a> {
    pub current_time: Duration,
    pub resources: &'a Resources,
    pub script_engine: &'a Engine,
    pub external_state: StateInterface<'a>,
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

/// All inputs provided to a node
pub struct Ins<'a> {
    oscs: &'a [&'a [UnsafeCell<Option<OscIndex>>]],
    values: &'a [&'a [UnsafeCell<Primitive>]],
    streams: &'a [&'a [&'a [UnsafeCell<f32>]]],
}

/// All outputs provided to a node
pub struct Outs<'a> {
    oscs: &'a [&'a [UnsafeCell<Option<OscIndex>>]],
    values: &'a [&'a [UnsafeCell<Primitive>]],
    streams: &'a [&'a [&'a [UnsafeCell<f32>]]],
}

// after this line is all of the IO api
pub struct InputOscSocket<'a> {
    oscs: &'a [UnsafeCell<Option<OscIndex>>],
}

pub struct InputValueSocket<'a> {
    values: &'a [UnsafeCell<Primitive>],
}

pub struct InputStreamSocket<'a> {
    streams: &'a [&'a [UnsafeCell<f32>]],
}

impl<'a> InputOscSocket<'a> {
    pub fn channel<'b>(&'b self, index: usize) -> &'b Option<OscIndex> {
        let osc = &self.oscs[index];

        unsafe { &*osc.get() }
    }

    pub fn iter<'b>(&'b self) -> impl Iterator<Item = &'b Option<OscIndex>> {
        self.oscs.iter().map(|osc| unsafe { &*osc.get() })
    }

    pub fn len(&self) -> usize {
        self.oscs.len()
    }
}

impl<'a> Index<usize> for InputOscSocket<'a> {
    type Output = Option<OscIndex>;

    fn index(&self, index: usize) -> &Self::Output {
        self.channel(index)
    }
}

impl<'a> InputValueSocket<'a> {
    pub fn channel<'b>(&'b self, index: usize) -> &'b Primitive {
        let value = &self.values[index];

        unsafe { &*value.get() }
    }

    pub fn iter<'b>(&'b self) -> impl Iterator<Item = &'b Primitive> {
        self.values.iter().map(|value| unsafe { &*value.get() })
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl<'a> Index<usize> for InputValueSocket<'a> {
    type Output = Primitive;

    fn index(&self, index: usize) -> &Self::Output {
        self.channel(index)
    }
}

impl<'a> InputStreamSocket<'a> {
    pub fn channel<'b>(&'b self, index: usize) -> &'b [f32] {
        let stream = self.streams[index];

        unsafe { mem::transmute::<&[UnsafeCell<f32>], &[f32]>(stream) }
    }

    pub fn iter<'b>(&'b self) -> impl Iterator<Item = &'b [f32]> {
        self.streams
            .iter()
            .map(|stream| unsafe { mem::transmute::<&[UnsafeCell<f32>], &[f32]>(stream) })
    }

    pub fn len(&self) -> usize {
        self.streams.len()
    }
}

impl<'a> Index<usize> for InputStreamSocket<'a> {
    type Output = [f32];

    fn index(&self, index: usize) -> &Self::Output {
        self.channel(index)
    }
}

impl<'a> Ins<'a> {
    pub unsafe fn new(
        oscs: &'a [&'a [UnsafeCell<Option<OscIndex>>]],
        values: &'a [&'a [UnsafeCell<Primitive>]],
        streams: &'a [&'a [&'a [UnsafeCell<f32>]]],
    ) -> Ins<'a> {
        Ins {
            oscs: oscs,
            values,
            streams,
        }
    }

    /// Get the osc socket at index `index`
    pub fn osc<'b>(&'b self, index: usize) -> InputOscSocket<'b> {
        InputOscSocket { oscs: self.oscs[index] }
    }

    /// Get the value socket at index `index`
    pub fn value<'b>(&'b self, index: usize) -> InputValueSocket<'b> {
        InputValueSocket {
            values: self.values[index],
        }
    }

    /// Get the stream socket at index `index`
    pub fn stream<'b>(&'b self, index: usize) -> InputStreamSocket<'b> {
        InputStreamSocket {
            streams: self.streams[index],
        }
    }

    pub fn oscs<'b>(&'b self) -> impl Iterator<Item = InputOscSocket<'b>> {
        self.oscs.iter().map(|osc| InputOscSocket { oscs: *osc })
    }

    pub fn values<'b>(&'b self) -> impl Iterator<Item = InputValueSocket<'b>> {
        self.values.iter().map(|value| InputValueSocket { values: *value })
    }

    pub fn streams<'b>(&'b self) -> impl Iterator<Item = InputStreamSocket<'b>> {
        self.streams.iter().map(|stream| InputStreamSocket { streams: *stream })
    }

    pub fn oscs_len(&self) -> usize {
        self.oscs.len()
    }

    pub fn values_len(&self) -> usize {
        self.values.len()
    }

    pub fn streams_len(&self) -> usize {
        self.streams.len()
    }
}

pub struct OutputOscSocket<'a> {
    pub oscs: &'a [UnsafeCell<Option<OscIndex>>],
}

pub struct OutputValueSocket<'a> {
    pub values: &'a [UnsafeCell<Primitive>],
}

pub struct OutputStreamSocket<'a> {
    pub streams: &'a [&'a [UnsafeCell<f32>]],
}

impl<'a> OutputOscSocket<'a> {
    pub fn channel<'b>(&'b mut self, index: usize) -> &'b mut Option<OscIndex> {
        let osc = &self.oscs[index];

        unsafe { &mut *osc.get() }
    }

    pub fn iter_mut<'b>(&'b mut self) -> impl Iterator<Item = &'b mut Option<OscIndex>> {
        self.oscs.iter().map(|osc| unsafe { &mut *osc.get() })
    }

    pub fn len(&self) -> usize {
        self.oscs.len()
    }
}

impl<'a> Index<usize> for OutputOscSocket<'a> {
    type Output = Option<OscIndex>;

    fn index<'b>(&'b self, index: usize) -> &'b Self::Output {
        let osc = &self.oscs[index];

        unsafe { &*osc.get() }
    }
}

impl<'a> IndexMut<usize> for OutputOscSocket<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.channel(index)
    }
}

impl<'a> OutputValueSocket<'a> {
    pub fn channel<'b>(&'b mut self, index: usize) -> &'b mut Primitive {
        let value = &self.values[index];

        unsafe { &mut *value.get() }
    }

    pub fn iter_mut<'b>(&'b mut self) -> impl Iterator<Item = &'b mut Primitive> {
        self.values.iter().map(|value| unsafe { &mut *value.get() })
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl<'a> Index<usize> for OutputValueSocket<'a> {
    type Output = Primitive;

    fn index<'b>(&'b self, index: usize) -> &'b Self::Output {
        let value = &self.values[index];

        unsafe { &*value.get() }
    }
}

impl<'a> IndexMut<usize> for OutputValueSocket<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.channel(index)
    }
}

impl<'a> OutputStreamSocket<'a> {
    pub fn channel<'b>(&'b mut self, index: usize) -> &'b mut [f32] {
        let stream = self.streams[index];

        unsafe { &mut *mem::transmute::<&[UnsafeCell<f32>], &UnsafeCell<[f32]>>(stream).get() }
    }

    pub fn iter_mut<'b>(&'b mut self) -> impl Iterator<Item = &'b mut [f32]> {
        self.streams
            .iter()
            .map(|stream| unsafe { &mut *mem::transmute::<&[UnsafeCell<f32>], &UnsafeCell<[f32]>>(stream).get() })
    }

    pub fn len(&self) -> usize {
        self.streams.len()
    }
}

impl<'a> Index<usize> for OutputStreamSocket<'a> {
    type Output = [f32];

    fn index<'b>(&'b self, index: usize) -> &'b Self::Output {
        let stream = self.streams[index];

        unsafe { mem::transmute::<&[UnsafeCell<f32>], &[f32]>(stream) }
    }
}

impl<'a> IndexMut<usize> for OutputStreamSocket<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.channel(index)
    }
}

impl<'a> Outs<'a> {
    pub unsafe fn new(
        oscs: &'a [&'a [UnsafeCell<Option<OscIndex>>]],
        values: &'a [&'a [UnsafeCell<Primitive>]],
        streams: &'a [&'a [&'a [UnsafeCell<f32>]]],
    ) -> Outs<'a> {
        Outs { oscs, values, streams }
    }

    /// Get the osc socket at index `index`
    pub fn osc<'b>(&'b mut self, index: usize) -> OutputOscSocket<'b> {
        OutputOscSocket { oscs: self.oscs[index] }
    }

    /// Get the value socket at index `index`
    pub fn value<'b>(&'b mut self, index: usize) -> OutputValueSocket<'b> {
        OutputValueSocket {
            values: self.values[index],
        }
    }

    /// Get the stream socket at index `index`
    pub fn stream<'b>(&'b mut self, index: usize) -> OutputStreamSocket<'b> {
        OutputStreamSocket {
            streams: self.streams[index],
        }
    }

    pub fn oscs<'b>(&'b mut self) -> impl Iterator<Item = OutputOscSocket<'b>> {
        self.oscs.iter().map(|osc| OutputOscSocket { oscs: *osc })
    }

    pub fn values<'b>(&'b mut self) -> impl Iterator<Item = OutputValueSocket<'b>> {
        self.values.iter().map(|value| OutputValueSocket { values: *value })
    }

    pub fn streams<'b>(&'b mut self) -> impl Iterator<Item = OutputStreamSocket<'b>> {
        self.streams
            .iter()
            .map(|stream| OutputStreamSocket { streams: *stream })
    }

    pub fn oscs_len(&self) -> usize {
        self.oscs.len()
    }

    pub fn values_len(&self) -> usize {
        self.values.len()
    }

    pub fn streams_len(&self) -> usize {
        self.streams.len()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct OscIndex(pub(super) generational_arena::Index);

impl OscIndex {
    pub(super) fn private_clone(&self) -> OscIndex {
        OscIndex(self.0)
    }
}

pub trait OptionExt {
    fn get_messages<'a>(&self, osc_store: &'a OscStore) -> Option<&'a [u8]>;
}

impl OptionExt for Option<OscIndex> {
    fn get_messages<'a>(&self, osc_store: &'a OscStore) -> Option<&'a [u8]> {
        self.as_ref().and_then(|index| osc_store.borrow_osc(&index))
    }
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
/// `set_state`, `accept_osc_inputs`, and `accept_value_inputs` are called before `process`
/// in an arbitrary order. They are also _only_ called if there is new incoming state.
///
/// `process` is called after that, and this is where most of the work happens.
///
/// After that, `get_osc_outputs`, `get_value_outputs` are called every time, and
/// that's how values are returned out. `get_state` is also called at this time
/// if the node has state to track.
///
/// To wrap up, `finish` is called at the end of processing.
#[allow(unused_variables)]
#[allow(unused_mut)]
#[enum_dispatch(NodeVariant)]
pub trait NodeRuntime: Debug {
    /// Called to initialize the state
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        InitResult::nothing()
    }

    /// Called once to check if it's stateful (TODO: move to static method)
    fn has_state(&self) -> bool {
        false
    }

    fn get_state(&self) -> Option<NodeState> {
        None
    }

    fn set_state(&mut self, state: serde_json::Value) {}

    /// reset params to initial state
    fn reset(&mut self) {}

    /// Process all data in and out
    fn process<'a>(
        &mut self,
        context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        osc_store: &mut OscStore,
        resources: &[Resource],
    ) {
    }
}

/// A static method returning a node's IO list. Note this is dynamic, but it cannot be dependent on
/// internal state
pub trait Node: NodeRuntime {
    /// Called at least every time a property is changed
    fn get_io(context: NodeGetIoContext, props: SeaHashMap<String, Property>) -> NodeIo;

    /// Called when created
    fn new(sound_config: &SoundConfig) -> Self;
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct NodeIndex(pub VertexIndex);

impl Debug for NodeIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.0)
    }
}
