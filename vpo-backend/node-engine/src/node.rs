//! Node module

use std::collections::HashMap;
use std::fmt::{Debug, Display};

use ddgg::VertexIndex;
use enum_dispatch::enum_dispatch;
use rhai::Engine;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{json, Value};
use smallvec::SmallVec;
use snafu::ResultExt;

use crate::connection::{
    MidiBundle, MidiSocketType, NodeRefSocketType, Primitive, SocketDirection, SocketType, StreamSocketType,
    ValueSocketType,
};

use crate::errors::{JsonParserSnafu, NodeError, NodeOk, NodeResult};
use crate::global_state::GlobalState;
use crate::graph_manager::{GraphIndex, GraphManager};
use crate::node_graph::NodeGraph;
use crate::nodes::inputs::InputsNode;
use crate::nodes::outputs::OutputsNode;
use crate::nodes::placeholder::Placeholder;
use crate::nodes::variants::{variant_to_name, NodeVariant};
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
    fn process(&mut self, state: NodeProcessState) -> Result<NodeOk<()>, NodeError> {
        NodeOk::no_warnings(())
    }

    /// Accept incoming stream data of type `socket_type`
    fn accept_stream_input(&mut self, socket_type: StreamSocketType, value: f32) {}

    /// Return outgoing stream data of type `socket_type`
    fn get_stream_output(&self, socket_type: StreamSocketType) -> f32 {
        0_f32
    }

    /// Accept incoming midi data of type `socket_type`
    fn accept_midi_input(&mut self, socket_type: MidiSocketType, value: MidiBundle) {}

    /// Return outgoing midi data of type `socket_type`
    fn get_midi_output(&self, socket_type: MidiSocketType) -> Option<MidiBundle> {
        Some(SmallVec::new())
    }

    /// Accept incoming value data of type `socket_type`
    fn accept_value_input(&mut self, socket_type: ValueSocketType, value: Primitive) {}

    /// Return outgoing value data of type `socket_type`
    fn get_value_output(&self, socket_type: ValueSocketType) -> Option<Primitive> {
        None
    }
}

fn serialize_node_prop<S>(node: &NodeVariant, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&variant_to_name(node))
}

fn deserialize_node_prop<'de, D>(deserializer: D) -> Result<NodeVariant, D::Error>
where
    D: Deserializer<'de>,
{
    let variant_name: String = serde::Deserialize::deserialize(deserializer)?;

    Ok(NodeVariant::Placeholder(Placeholder::new(variant_name)))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeWrapper {
    #[serde(serialize_with = "serialize_node_prop")]
    #[serde(deserialize_with = "deserialize_node_prop")]
    pub(crate) node: NodeVariant,
    node_rows: Vec<NodeRow>,
    default_overrides: Vec<NodeRow>,
    properties: HashMap<String, Property>,
    ui_data: HashMap<String, Value>,
    child_graph_index: Option<GraphIndex>,
    child_graph_io_indexes: Option<(NodeIndex, NodeIndex)>,
}

impl NodeWrapper {
    pub fn new(mut node: NodeVariant, state: NodeInitState) -> Result<NodeOk<NodeWrapper>, NodeError> {
        let name = variant_to_name(&node);

        let init_result = node.init(state)?;
        // TODO: check validity of node_rows here (no socket duplicates)

        // extract properties from result from `init`
        // this fills the properties with the default values
        let properties = init_result
            .value
            .node_rows
            .iter()
            .filter_map(|row| match row {
                NodeRow::Property(name, _, default) => Some((name, default)),
                _ => None,
            })
            .fold(HashMap::new(), |mut accum, (name, default)| {
                accum.insert(name.clone(), default.clone());
                accum
            });

        let mut wrapper = NodeWrapper {
            node,
            default_overrides: Vec::new(),
            node_rows: init_result.value.node_rows,
            properties,
            ui_data: HashMap::new(),
            child_graph_index: None,
            child_graph_io_indexes: None,
        };

        // insert some initial UI data
        wrapper.ui_data.insert("x".to_string(), json! { 0.0_f32 });
        wrapper.ui_data.insert("y".to_string(), json! { 0.0_f32 });

        wrapper.ui_data.insert("title".to_string(), json! { name });

        Ok(NodeOk {
            value: wrapper,
            warnings: init_result.warnings,
        })
    }

    pub fn uses_child_graph(&self) -> bool {
        self.node_rows.iter().any(|row| matches!(row, NodeRow::InnerGraph))
    }

    pub fn init_child_graph(
        &mut self,
        index: GraphIndex,
        graph_manager: &GraphManager,
        inputs: Vec<SocketType>,
        outputs: Vec<SocketType>,
        state: NodeInitState,
    ) -> Result<(), NodeError> {
        self.set_child_graph_index(index);

        let mut new_inputs_node = InputsNode::default();
        let mut new_outputs_node = OutputsNode::default();

        new_inputs_node.set_inputs(inputs);
        new_outputs_node.set_outputs(outputs);

        let child_graph = &mut graph_manager.get_graph(index)?.graph.borrow_mut();

        let NodeInitState {
            props,
            registry,
            script_engine,
            global_state,
        } = state;

        let (input_index, _) = child_graph
            .add_node(
                NodeVariant::InputsNode(new_inputs_node),
                NodeInitState {
                    props,
                    registry,
                    script_engine,
                    global_state,
                },
            )
            .unwrap()
            .value;
        let (output_index, _) = child_graph
            .add_node(
                NodeVariant::OutputsNode(new_outputs_node),
                NodeInitState {
                    props,
                    registry,
                    script_engine,
                    global_state,
                },
            )
            .unwrap()
            .value;

        self.child_graph_io_indexes = Some((input_index, output_index));

        Ok(())
    }

    pub fn set_child_graph_index(&mut self, index: GraphIndex) {
        self.child_graph_index = Some(index);
    }

    pub fn get_child_graph_index(&self) -> &Option<GraphIndex> {
        &self.child_graph_index
    }

    pub fn get_property(&self, name: &str) -> Option<Property> {
        self.properties.get(name).cloned()
    }

    pub fn set_property(&mut self, name: String, value: Property) {
        self.properties.insert(name, value);
    }

    pub fn get_properties(&self) -> &HashMap<String, Property> {
        &self.properties
    }

    pub fn set_properties(&mut self, properties: HashMap<String, Property>) {
        self.properties = properties;
    }

    pub fn replace_properties(&mut self, properties: HashMap<String, Property>) -> HashMap<String, Property> {
        std::mem::replace(&mut self.properties, properties)
    }

    pub fn get_ui_data(&self) -> &HashMap<String, Value> {
        &self.ui_data
    }

    pub fn set_ui_data(&mut self, ui_data: HashMap<String, Value>) {
        self.ui_data = ui_data;
    }

    pub fn replace_ui_data(&mut self, ui_data: HashMap<String, Value>) -> HashMap<String, Value> {
        std::mem::replace(&mut self.ui_data, ui_data)
    }

    pub fn set_default_overrides(&mut self, default_overrides: Vec<NodeRow>) {
        self.default_overrides = default_overrides;
    }

    pub fn replace_default_overrides(&mut self, default_overrides: Vec<NodeRow>) -> Vec<NodeRow> {
        std::mem::replace(&mut self.default_overrides, default_overrides)
    }

    pub fn set_ui_data_property(&mut self, key: String, value: Value) {
        self.ui_data.insert(key, value);
    }

    pub fn list_input_sockets(&self) -> Vec<SocketType> {
        self.node_rows
            .iter()
            .filter_map(|row| match row {
                NodeRow::StreamInput(stream_input_type, ..) => Some(SocketType::Stream(stream_input_type.clone())),
                NodeRow::MidiInput(midi_input_type, ..) => Some(SocketType::Midi(midi_input_type.clone())),
                NodeRow::ValueInput(value_input_type, ..) => Some(SocketType::Value(value_input_type.clone())),
                NodeRow::NodeRefInput(node_ref_input_type, ..) => {
                    Some(SocketType::NodeRef(node_ref_input_type.clone()))
                }
                NodeRow::StreamOutput(..) => None,
                NodeRow::MidiOutput(..) => None,
                NodeRow::ValueOutput(..) => None,
                NodeRow::NodeRefOutput(..) => None,
                NodeRow::Property(..) => None,
                NodeRow::InnerGraph => None,
            })
            .collect()
    }

    pub fn list_output_sockets(&self) -> Vec<SocketType> {
        self.node_rows
            .iter()
            .filter_map(|row| match row {
                NodeRow::StreamInput(..) => None,
                NodeRow::MidiInput(..) => None,
                NodeRow::ValueInput(..) => None,
                NodeRow::NodeRefInput(..) => None,
                NodeRow::StreamOutput(stream_output_type, ..) => Some(SocketType::Stream(stream_output_type.clone())),
                NodeRow::MidiOutput(midi_output_type, ..) => Some(SocketType::Midi(midi_output_type.clone())),
                NodeRow::ValueOutput(value_output_type, ..) => Some(SocketType::Value(value_output_type.clone())),
                NodeRow::NodeRefOutput(node_ref_output_type, ..) => {
                    Some(SocketType::NodeRef(node_ref_output_type.clone()))
                }
                NodeRow::Property(..) => None,
                NodeRow::InnerGraph => None,
            })
            .collect()
    }

    pub fn has_input_socket(&self, socket_type: SocketType) -> bool {
        self.list_input_sockets().iter().any(|&socket| socket == socket_type)
    }

    pub fn has_output_socket(&self, socket_type: SocketType) -> bool {
        self.list_output_sockets().iter().any(|&socket| socket == socket_type)
    }

    pub fn get_default(&self, socket_type: &SocketType) -> Option<NodeRow> {
        let possible_override = self.default_overrides.iter().find(|override_row| {
            let type_and_direction = (*override_row).clone().to_type_and_direction();

            if let Some((override_type, override_direction)) = type_and_direction {
                socket_type == &override_type && SocketDirection::Input == override_direction
            } else {
                false
            }
        });

        if let Some(row_override) = possible_override {
            return Some(row_override.clone());
        }

        self.node_rows
            .iter()
            .find(|node_row| {
                let type_and_direction = (*node_row).clone().to_type_and_direction();

                if let Some((override_type, override_direction)) = type_and_direction {
                    socket_type == &override_type && SocketDirection::Input == override_direction
                } else {
                    false
                }
            })
            .cloned()
    }

    /// Note, this does not deserialize the node itself, only the generic properties
    pub fn apply_json(&mut self, json: &mut Value) -> Result<(), NodeError> {
        println!("Applying json: {}", json);

        let ui_data: HashMap<String, Value> = serde_json::from_value(json["uiData"].take()).context(JsonParserSnafu)?;

        self.ui_data = ui_data;

        Ok(())
    }

    pub fn accept_stream_input(&mut self, socket_type: StreamSocketType, value: f32) {
        self.node.accept_stream_input(socket_type, value);
    }

    pub fn get_stream_output(&self, socket_type: StreamSocketType) -> f32 {
        self.node.get_stream_output(socket_type)
    }

    pub fn accept_midi_input(&mut self, socket_type: MidiSocketType, value: MidiBundle) {
        self.node.accept_midi_input(socket_type, value);
    }

    pub fn get_midi_output(&self, socket_type: MidiSocketType) -> Option<MidiBundle> {
        self.node.get_midi_output(socket_type)
    }

    pub fn accept_value_input(&mut self, socket_type: ValueSocketType, value: Primitive) {
        self.node.accept_value_input(socket_type, value);
    }

    pub fn get_value_output(&self, socket_type: ValueSocketType) -> Option<Primitive> {
        self.node.get_value_output(socket_type)
    }

    pub fn process(&mut self, state: NodeProcessState) -> Result<NodeOk<()>, NodeError> {
        self.node.process(state)
    }

    pub fn get_child_graph_socket_list(&self, registry: &mut SocketRegistry) -> Vec<(SocketType, SocketDirection)> {
        self.node.get_child_graph_socket_list(registry)
    }

    pub fn node_init_graph(&mut self, graph: &mut NodeGraph) {
        let (input_node, output_node) = &self.child_graph_io_indexes.unwrap();

        self.node.init_graph(graph, *input_node, *output_node);
    }

    pub fn get_node_type(&self) -> String {
        variant_to_name(&self.node)
    }

    pub fn linked_to_ui(&self) -> bool {
        self.node.linked_to_ui()
    }

    pub(crate) fn get_child_graph_io_indexes(&self) -> &Option<(NodeIndex, NodeIndex)> {
        &self.child_graph_io_indexes
    }

    pub(crate) fn set_node_rows(&mut self, rows: Vec<NodeRow>) {
        self.node_rows = rows;
    }

    pub(crate) fn get_node_rows(&self) -> &Vec<NodeRow> {
        &self.node_rows
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct NodeIndex(pub VertexIndex);

impl Display for NodeIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.0)
    }
}
