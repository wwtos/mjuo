//! Node module

use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::{cell::RefCell, rc::Rc};

use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use sound_engine::midi::messages::MidiData;

use crate::connection::{InputSideConnection, OutputSideConnection, SocketType, StreamSocketType, MidiSocketType, ValueSocketType, Parameter};

use crate::errors::NodeError;
use crate::nodes::variants::{NodeVariant, variant_to_name};
use crate::property::{PropertyType, Property};

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
pub trait Node: Debug {
    // defaults list nothing, to reduce boilerplate necessary for
    // nodes that don't use all node functionality
    fn list_input_sockets(&self) -> Vec<SocketType> {
        Vec::new()
    }

    fn list_output_sockets(&self) -> Vec<SocketType> {
        Vec::new()
    }

    fn list_properties(&self) -> HashMap<String, PropertyType> {
        HashMap::new()
    }

    fn process(&mut self) {}

    fn accept_stream_input(&mut self, socket_type: StreamSocketType, value: f32) {}

    fn get_stream_output(&self, socket_type: StreamSocketType) -> f32 {
        0_f32
    }

    fn accept_midi_input(&mut self, socket_type: MidiSocketType, value: Vec<MidiData>) {}

    fn get_midi_output(&self, socket_type: MidiSocketType) -> Vec<MidiData> {
        vec![]
    }

    fn accept_value_input(&mut self, socket_type: ValueSocketType, value: Parameter) {}

    fn get_value_output(&self, socket_type: ValueSocketType) -> Option<Parameter> {
        None
    }
}

#[derive(Debug, Serialize)]
pub struct NodeWrapper {
    pub(crate) node: NodeVariant,
    index: NodeIndex,
    connected_inputs: Vec<InputSideConnection>,
    connected_outputs: Vec<OutputSideConnection>,
    properties: HashMap<String, Property>,
    ui_data: HashMap<String, Value>
}

impl NodeWrapper {
    pub fn new(node: NodeVariant, index: NodeIndex) -> NodeWrapper {
        let name = variant_to_name(&node);

        let mut wrapper = NodeWrapper {
            node,
            index,
            connected_inputs: Vec::new(),
            connected_outputs: Vec::new(),
            properties: HashMap::new(),
            ui_data: HashMap::new()
        };

        wrapper.ui_data.insert("x".to_string(), json! { 0.0 });
        wrapper.ui_data.insert("y".to_string(), json! { 0.0 });

        wrapper.ui_data.insert("title".to_string(), json! { name });

        wrapper
    }

    pub fn get_index(&self) -> NodeIndex {
        self.index
    }

    pub fn get_property(&self, name: &str) -> Option<Property> {
        self.properties.get(name).map(|prop| prop.clone())
    }

    pub fn set_property(&mut self, name: String, value: Property) {
        self.properties.insert(name, value);
    }
 
    pub fn list_input_sockets(&self) -> Vec<InputSideConnection> {
        self.connected_inputs.clone()
    }

    pub fn list_output_sockets(&self) -> Vec<OutputSideConnection> {
        self.connected_outputs.clone()
    }

    pub fn has_input_socket(&self, socket_type: &SocketType) -> bool {
        self.node
            .as_ref()
            .list_input_sockets()
            .iter()
            .any(|socket| *socket == *socket_type)
    }

    pub fn has_output_socket(&self, socket_type: &SocketType) -> bool {
        self.node
            .as_ref()
            .list_output_sockets()
            .iter()
            .any(|socket| *socket == *socket_type)
    }

    pub fn get_input_connection_by_type(
        &self,
        input_socket_type: &SocketType,
    ) -> Option<InputSideConnection> {
        let input = self
            .connected_inputs
            .iter()
            .find(|input| input.to_socket_type == *input_socket_type);

        input.map(|input| (*input).clone())
    }

    pub fn get_output_connections_by_type(
        &self,
        output_socket_type: &SocketType,
    ) -> Vec<OutputSideConnection> {
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

    pub fn serialize_to_json(&self) -> Result<serde_json::Value, NodeError> {
        Ok(json! {{
            "node": {
                "input_sockets": self.node.as_ref().list_input_sockets(),
                "output_sockets": self.node.as_ref().list_output_sockets(),
                "properties": self.node.as_ref().list_properties()
            },
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

        if index != self.index {
            return Err(NodeError::MismatchedNodeIndex(self.index, index));
        }

        self.properties = properties;
        self.ui_data = ui_data;

        Ok(())
    }

    pub fn accept_stream_input(&mut self, socket_type: StreamSocketType, value: f32) {
        self.node.as_mut().accept_stream_input(socket_type, value);
    }

    pub fn get_stream_output(&self, socket_type: StreamSocketType) -> f32 {
        self.node.as_ref().get_stream_output(socket_type)
    }

    pub fn accept_midi_input(&mut self, socket_type: MidiSocketType, value: Vec<MidiData>) {
        self.node.as_mut().accept_midi_input(socket_type, value);
    }

    pub fn get_midi_output(&self, socket_type: MidiSocketType) -> Vec<MidiData> {
        self.node.as_ref().get_midi_output(socket_type)
    }

    pub fn accept_value_input(&mut self, socket_type: ValueSocketType, value: Parameter) {
        self.node.as_mut().accept_value_input(socket_type, value);
    }

    pub fn get_value_output(&self, socket_type: ValueSocketType) -> Option<Parameter> {
        self.node.as_ref().get_value_output(socket_type)
    }

    pub fn process(&mut self) {
        self.node.as_mut().process();
    }

    pub(in crate) fn set_index(&mut self, index: NodeIndex) {
        self.index = index;
    }

    pub(in crate) fn add_input_connection_unsafe(&mut self, connection: InputSideConnection) {
        self.connected_inputs.push(connection);
    }

    pub(in crate) fn add_output_connection_unsafe(&mut self, connection: OutputSideConnection) {
        self.connected_outputs.push(connection);
    }

    pub(in crate) fn remove_input_socket_connection_unsafe(&mut self, to_type: &SocketType) -> Result<(), NodeError> {
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

    pub(in crate) fn remove_output_socket_connection_unsafe(
        &mut self,
        connection: &OutputSideConnection
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

    pub(in crate) fn remove_output_socket_connections_unsafe(
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

#[derive(Debug, Clone)]
pub struct GenerationalNode {
    pub node: Rc<RefCell<NodeWrapper>>,
    pub generation: u32,
}
