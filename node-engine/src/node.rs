use std::fmt::Debug;
use std::{cell::RefCell, rc::Rc};

use crate::connection::{SocketType, StreamSocketType, InputSideConnection, OutputSideConnection};

use crate::errors::{Error, ErrorType};

pub trait Node: Debug {
    fn list_input_sockets(&self) -> Vec<SocketType>;
    fn list_output_sockets(&self) -> Vec<SocketType>;
    fn accept_stream_input(&mut self, socket_type: StreamSocketType, value: f32);
    fn get_stream_output(&mut self, socket_type: StreamSocketType) -> f32;
    fn serialize_to_json(&self) -> Result<serde_json::Value, Error>;
    fn deserialize_from_json(json: serde_json::Value) -> Self
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct NodeWrapper {
    pub(crate) node: Box<dyn Node>,
    index: NodeIndex,
    connected_inputs: Vec<InputSideConnection>,
    connected_outputs: Vec<OutputSideConnection>,
}

impl NodeWrapper {
    pub fn new(node: Box<dyn Node>, index: NodeIndex) -> NodeWrapper {
        NodeWrapper {
            node,
            index,
            connected_inputs: Vec::new(),
            connected_outputs: Vec::new(),
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
            .list_input_sockets()
            .iter()
            .find(|socket| *socket == socket_type)
            .is_some()
    }

    pub fn has_output_socket(&self, socket_type: &SocketType) -> bool {
        self.node
            .list_output_sockets()
            .iter()
            .find(|socket| *socket == socket_type)
            .is_some()
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

    pub fn remove_input_socket_connection(&mut self, to_type: &SocketType) -> Result<(), Error> {
        let to_remove = self
            .connected_inputs
            .iter()
            .position(|input| input.to_socket_type == *to_type);

        if let Some(to_remove) = to_remove {
            self.connected_inputs.remove(to_remove);

            Ok(())
        } else {
            Err(Error::new(
                "Connection doesn't exist!".to_string(),
                ErrorType::NotConnected,
            ))
        }
    }

    pub fn remove_output_socket_connection(
        &mut self,
        from_type: &SocketType,
        to_node: &NodeIndex,
        to_type: &SocketType,
    ) -> Result<(), Error> {
        let to_remove = self.connected_outputs.iter().position(|input| {
            input.from_socket_type == *from_type
                && input.to_node == *to_node
                && input.to_socket_type == *to_type
        });

        if let Some(to_remove) = to_remove {
            self.connected_outputs.remove(to_remove);

            Ok(())
        } else {
            Err(Error::new(
                "Connection doesn't exist!".to_string(),
                ErrorType::NotConnected,
            ))
        }
    }

    pub fn remove_output_socket_connections(
        &mut self,
        from_type: &SocketType,
    ) -> Result<(), Error> {
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
            Err(Error::new(
                "Connection doesn't exist!".to_string(),
                ErrorType::NotConnected,
            ))
        } else {
            Ok(())
        }
    }

    pub fn serialize_to_json(&self) -> Result<serde_json::Value, Error> {
        self.node.serialize_to_json()
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct NodeIndex {
    pub index: usize,
    pub generation: u32,
}

#[derive(Debug, Clone)]
pub struct GenerationalNode {
    pub node: Rc<RefCell<NodeWrapper>>,
    pub generation: u32,
}
