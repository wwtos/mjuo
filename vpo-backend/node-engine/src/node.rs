//! Node module

use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::{cell::RefCell, rc::Rc};

use serde::{Serialize, Deserialize};

use crate::connection::{InputSideConnection, OutputSideConnection, SocketType, StreamSocketType};

use crate::errors::NodeError;
use crate::nodes::variants::NodeVariant;
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

    fn accept_stream_input(&mut self, socket_type: StreamSocketType, value: f32) {}

    fn get_stream_output(&mut self, socket_type: StreamSocketType) -> f32 {
        0_f32
    }
}

#[derive(Debug)]
pub struct NodeWrapper {
    pub(crate) node: NodeVariant,
    index: NodeIndex,
    connected_inputs: Vec<InputSideConnection>,
    connected_outputs: Vec<OutputSideConnection>,
    properties: HashMap<String, Property>
}

impl NodeWrapper {
    pub fn new(node: NodeVariant, index: NodeIndex) -> NodeWrapper {
        NodeWrapper {
            node,
            index,
            connected_inputs: Vec::new(),
            connected_outputs: Vec::new(),
            properties: HashMap::new()
        }
    }

    pub fn get_index(&self) -> NodeIndex {
        self.index
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

    pub fn remove_input_socket_connection(&mut self, to_type: &SocketType) -> Result<(), NodeError> {
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

    pub fn remove_output_socket_connection(
        &mut self,
        from_type: &SocketType,
        to_node: &NodeIndex,
        to_type: &SocketType,
    ) -> Result<(), NodeError> {
        let to_remove = self.connected_outputs.iter().position(|input| {
            input.from_socket_type == *from_type
                && input.to_node == *to_node
                && input.to_socket_type == *to_type
        });

        if let Some(to_remove) = to_remove {
            self.connected_outputs.remove(to_remove);

            Ok(())
        } else {
            Err(NodeError::NotConnected)
        }
    }

    pub fn remove_output_socket_connections(
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

    pub fn serialize_to_json(&self) -> Result<serde_json::Value, NodeError> {
        Ok(serde_json::to_value(&self.node)?)
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
