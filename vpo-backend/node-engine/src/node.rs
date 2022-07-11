//! Node module

use std::collections::HashMap;
use std::fmt::{Debug, Display};

use enum_dispatch::enum_dispatch;
use rhai::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sound_engine::midi::messages::MidiData;

use crate::connection::{
    InputSideConnection, MidiSocketType, NodeRefSocketType, OutputSideConnection, Primitive, SocketDirection,
    SocketType, StreamSocketType, ValueSocketType,
};

use crate::errors::{ErrorsAndWarnings, NodeError};
use crate::nodes::variants::{variant_to_name, NodeVariant};
use crate::property::{Property, PropertyType};
use crate::socket_registry::SocketRegistry;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "variant", content = "data")]
pub enum NodeRow {
    StreamInput(StreamSocketType, f32),
    MidiInput(MidiSocketType, Vec<MidiData>),
    ValueInput(ValueSocketType, Primitive),
    NodeRefInput(NodeRefSocketType),
    StreamOutput(StreamSocketType, f32),
    MidiOutput(MidiSocketType, Vec<MidiData>),
    ValueOutput(ValueSocketType, Primitive),
    NodeRefOutput(NodeRefSocketType),
    Property(String, PropertyType, Property),
    InnerGraph,
}

impl NodeRow {
    pub fn to_type_and_direction(self) -> Option<(SocketType, SocketDirection)> {
        match self {
            NodeRow::StreamInput(stream_type, _) => Some((SocketType::Stream(stream_type), SocketDirection::Input)),
            NodeRow::MidiInput(midi_type, _) => Some((SocketType::Midi(midi_type), SocketDirection::Input)),
            NodeRow::ValueInput(value_type, _) => Some((SocketType::Value(value_type), SocketDirection::Input)),
            NodeRow::NodeRefInput(node_ref_type) => Some((SocketType::NodeRef(node_ref_type), SocketDirection::Input)),
            NodeRow::StreamOutput(stream_type, _) => Some((SocketType::Stream(stream_type), SocketDirection::Output)),
            NodeRow::MidiOutput(midi_type, _) => Some((SocketType::Midi(midi_type), SocketDirection::Output)),
            NodeRow::ValueOutput(value_type, _) => Some((SocketType::Value(value_type), SocketDirection::Output)),
            NodeRow::NodeRefOutput(node_ref_type) => {
                Some((SocketType::NodeRef(node_ref_type), SocketDirection::Output))
            }
            NodeRow::Property(..) => None,
            NodeRow::InnerGraph => None,
        }
    }
}

pub struct InitResult {
    pub did_rows_change: bool,
    pub node_rows: Vec<NodeRow>,
    pub changed_properties: Option<HashMap<String, Property>>,
    pub errors_and_warnings: Option<ErrorsAndWarnings>,
}

impl InitResult {
    pub fn simple(node_rows: Vec<NodeRow>) -> InitResult {
        InitResult {
            did_rows_change: false,
            node_rows,
            changed_properties: None,
            errors_and_warnings: None,
        }
    }
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
pub trait Node: Debug {
    fn init(
        &mut self,
        props: &HashMap<String, Property>,
        registry: &mut SocketRegistry,
        scripting_engine: &Engine,
    ) -> InitResult;

    /// Process received data.
    fn process(&mut self, current_time: i64, scripting_engine: &Engine) -> Result<(), ErrorsAndWarnings> {
        Ok(())
    }

    /// Accept incoming stream data of type `socket_type`
    fn accept_stream_input(&mut self, socket_type: &StreamSocketType, value: f32) {}

    /// Return outgoing stream data of type `socket_type`
    fn get_stream_output(&self, socket_type: &StreamSocketType) -> f32 {
        0_f32
    }

    /// Accept incoming midi data of type `socket_type`
    fn accept_midi_input(&mut self, socket_type: &MidiSocketType, value: Vec<MidiData>) {}

    /// Return outgoing midi data of type `socket_type`
    fn get_midi_output(&self, socket_type: &MidiSocketType) -> Vec<MidiData> {
        vec![]
    }

    /// Accept incoming value data of type `socket_type`
    fn accept_value_input(&mut self, socket_type: &ValueSocketType, value: Primitive) {}

    /// Return outgoing value data of type `socket_type`
    fn get_value_output(&self, socket_type: &ValueSocketType) -> Option<Primitive> {
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeWrapper {
    #[serde(skip)]
    pub(crate) node: NodeVariant,
    index: NodeIndex,
    connected_inputs: Vec<InputSideConnection>,
    connected_outputs: Vec<OutputSideConnection>,
    node_rows: Vec<NodeRow>,
    default_overrides: Vec<NodeRow>,
    properties: HashMap<String, Property>,
    ui_data: HashMap<String, Value>,
}

impl NodeWrapper {
    pub fn new(
        mut node: NodeVariant,
        index: NodeIndex,
        registry: &mut SocketRegistry,
        scripting_engine: &Engine,
    ) -> NodeWrapper {
        let name = variant_to_name(&node);

        let init_result = node.init(&HashMap::new(), registry, scripting_engine);

        // TODO: check validity of node_rows here (no socket duplicates)

        // extract properties from result from `init`
        // this fills the properties with the default values
        let properties = init_result
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
            index,
            default_overrides: Vec::new(),
            connected_inputs: Vec::new(),
            connected_outputs: Vec::new(),
            node_rows: init_result.node_rows,
            properties,
            ui_data: HashMap::new(),
        };

        // insert some initial UI data
        wrapper.ui_data.insert("x".to_string(), json! { 0.0_f32 });
        wrapper.ui_data.insert("y".to_string(), json! { 0.0_f32 });

        wrapper.ui_data.insert("title".to_string(), json! { name });

        wrapper
    }

    pub fn get_index(&self) -> NodeIndex {
        self.index
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

    pub fn get_ui_data(&self) -> &HashMap<String, Value> {
        &self.ui_data
    }

    pub fn set_ui_data(&mut self, ui_data: HashMap<String, Value>) {
        self.ui_data = ui_data;
    }

    pub fn set_ui_data_property(&mut self, key: String, value: Value) {
        self.ui_data.insert(key, value);
    }

    pub fn list_connected_input_sockets(&self) -> Vec<InputSideConnection> {
        self.connected_inputs.clone()
    }

    pub fn list_connected_output_sockets(&self) -> Vec<OutputSideConnection> {
        self.connected_outputs.clone()
    }

    pub fn list_input_sockets(&self) -> Vec<SocketType> {
        self.node_rows
            .iter()
            .filter_map(|row| match row {
                NodeRow::StreamInput(stream_input_type, _) => Some(SocketType::Stream(stream_input_type.clone())),
                NodeRow::MidiInput(midi_input_type, _) => Some(SocketType::Midi(midi_input_type.clone())),
                NodeRow::ValueInput(value_input_type, _) => Some(SocketType::Value(value_input_type.clone())),
                NodeRow::NodeRefInput(node_ref_input_type) => Some(SocketType::NodeRef(node_ref_input_type.clone())),
                NodeRow::StreamOutput(_, _) => None,
                NodeRow::MidiOutput(_, _) => None,
                NodeRow::ValueOutput(_, _) => None,
                NodeRow::NodeRefOutput(_) => None,
                NodeRow::Property(..) => None,
                NodeRow::InnerGraph => None,
            })
            .collect()
    }

    pub fn list_output_sockets(&self) -> Vec<SocketType> {
        self.node_rows
            .iter()
            .filter_map(|row| match row {
                NodeRow::StreamInput(_, _) => None,
                NodeRow::MidiInput(_, _) => None,
                NodeRow::ValueInput(_, _) => None,
                NodeRow::NodeRefInput(_) => None,
                NodeRow::StreamOutput(stream_output_type, _) => Some(SocketType::Stream(stream_output_type.clone())),
                NodeRow::MidiOutput(midi_output_type, _) => Some(SocketType::Midi(midi_output_type.clone())),
                NodeRow::ValueOutput(value_output_type, _) => Some(SocketType::Value(value_output_type.clone())),
                NodeRow::NodeRefOutput(node_ref_output_type) => Some(SocketType::NodeRef(node_ref_output_type.clone())),
                NodeRow::Property(..) => None,
                NodeRow::InnerGraph => None,
            })
            .collect()
    }

    pub fn has_input_socket(&self, socket_type: &SocketType) -> bool {
        self.list_input_sockets().iter().any(|socket| *socket == *socket_type)
    }

    pub fn has_output_socket(&self, socket_type: &SocketType) -> bool {
        self.list_output_sockets().iter().any(|socket| *socket == *socket_type)
    }

    pub fn get_input_connection_by_type(&self, input_socket_type: &SocketType) -> Option<InputSideConnection> {
        let input = self
            .connected_inputs
            .iter()
            .find(|input| input.to_socket_type == *input_socket_type);

        input.map(|input| (*input).clone())
    }

    pub fn get_output_connections_by_type(&self, output_socket_type: &SocketType) -> Vec<OutputSideConnection> {
        let my_outputs_filtered = self
            .connected_outputs
            .iter()
            .filter(|input| input.from_socket_type == *output_socket_type);

        let mut outputs_filtered: Vec<OutputSideConnection> = Vec::new();

        for output in my_outputs_filtered {
            outputs_filtered.push((*output).clone());
        }

        outputs_filtered
    }

    pub fn get_default(&self, socket_type: &SocketType) -> Option<NodeRow> {
        // if it's connected to something, it doesn't have a default
        if let Some(_) = self.get_input_connection_by_type(socket_type) {
            return None;
        }

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
            .map(|row| row.clone())
    }

    pub fn serialize_to_json(&self) -> Result<serde_json::Value, NodeError> {
        Ok(json! {{
            "node_rows": self.node_rows.clone(),
            "default_overrides": self.default_overrides.clone(),
            "index": self.index,
            "connected_inputs": self.connected_inputs,
            "connected_outputs": self.connected_outputs,
            "properties": self.properties,
            "ui_data": self.ui_data
        }})
    }

    /// Note, this does not deserialize the node itself, only the generic properties
    pub fn apply_json(&mut self, json: &Value) -> Result<(), NodeError> {
        println!("Applying json: {}", json);

        let index: NodeIndex = serde_json::from_value(json["index"].clone())?;
        let properties: HashMap<String, Property> = serde_json::from_value(json["properties"].clone())?;
        let ui_data: HashMap<String, Value> = serde_json::from_value(json["ui_data"].clone())?;
        let default_overrides = serde_json::from_value(json["default_overrides"].clone())?;

        if index != self.index {
            return Err(NodeError::MismatchedNodeIndex(self.index, index));
        }

        self.properties = properties;
        self.default_overrides = default_overrides;
        self.ui_data = ui_data;

        Ok(())
    }

    pub fn accept_stream_input(&mut self, socket_type: &StreamSocketType, value: f32) {
        self.node.accept_stream_input(socket_type, value);
    }

    pub fn get_stream_output(&self, socket_type: &StreamSocketType) -> f32 {
        self.node.get_stream_output(socket_type)
    }

    pub fn accept_midi_input(&mut self, socket_type: &MidiSocketType, value: Vec<MidiData>) {
        self.node.accept_midi_input(socket_type, value);
    }

    pub fn get_midi_output(&self, socket_type: &MidiSocketType) -> Vec<MidiData> {
        self.node.get_midi_output(socket_type)
    }

    pub fn accept_value_input(&mut self, socket_type: &ValueSocketType, value: Primitive) {
        self.node.accept_value_input(socket_type, value);
    }

    pub fn get_value_output(&self, socket_type: &ValueSocketType) -> Option<Primitive> {
        self.node.get_value_output(socket_type)
    }

    pub fn process(&mut self, current_time: i64, scripting_engine: &Engine) -> Result<(), ErrorsAndWarnings> {
        self.node.process(current_time, scripting_engine)
    }

    pub(in crate) fn set_index(&mut self, index: NodeIndex) {
        self.index = index;
    }

    pub(in crate) fn set_node_rows(&mut self, rows: Vec<NodeRow>) {
        self.node_rows = rows;
    }

    pub(in crate) fn get_node_rows(&self) -> &Vec<NodeRow> {
        &self.node_rows
    }

    pub(in crate) fn get_output_connections(&self) -> &Vec<OutputSideConnection> {
        &self.connected_outputs
    }

    pub(in crate) fn add_input_connection_unchecked(&mut self, connection: InputSideConnection) {
        self.connected_inputs.push(connection);
    }

    pub(in crate) fn add_output_connection_unchecked(&mut self, connection: OutputSideConnection) {
        self.connected_outputs.push(connection);
    }

    pub(in crate) fn remove_input_socket_connection_unchecked(&mut self, to_type: &SocketType) -> Result<(), NodeError> {
        let to_remove = self
            .connected_inputs
            .iter()
            .position(|input| input.to_socket_type == *to_type);

        if let Some(to_remove) = to_remove {
            self.connected_inputs.remove(to_remove);

            Ok(())
        } else {
            Err(NodeError::NotConnected)
        }
    }

    pub(in crate) fn remove_output_socket_connection_unchecked(
        &mut self,
        connection: &OutputSideConnection,
    ) -> Result<(), NodeError> {
        let to_remove = self.connected_outputs.iter().position(|input| {
            input.from_socket_type == connection.from_socket_type
                && input.to_node == connection.to_node
                && input.to_socket_type == connection.to_socket_type
        });

        if let Some(to_remove) = to_remove {
            self.connected_outputs.remove(to_remove);

            Ok(())
        } else {
            Err(NodeError::NotConnected)
        }
    }

    pub(in crate) fn _remove_output_socket_connections_unchecked(
        &mut self,
        from_type: &SocketType,
    ) -> Result<(), NodeError> {
        let mut found: Vec<usize> = Vec::new();

        for (i, connection) in self.connected_outputs.iter().enumerate() {
            if connection.from_socket_type == *from_type {
                found.push(i);
            }
        }

        for found_index in &found {
            self.connected_inputs.remove(*found_index);
        }

        if found.is_empty() {
            Err(NodeError::NotConnected)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct NodeIndex {
    pub index: usize,
    pub generation: u32,
}

impl Display for NodeIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "index: {}, generation: {}", self.index, self.generation)
    }
}
