use std::{cell::RefCell, rc::Rc};

use sound_engine::midi::messages::MidiData;

pub trait Node {
    fn list_input_sockets(&self) -> Vec<SocketType>;
    fn list_output_sockets(&self) -> Vec<SocketType>;
}

pub struct NodeWrapper {
    node: Box<dyn Node>,
    index: NodeIndex,
    input: Option<Connection>,
    outputs: Vec<Connection>,
}

impl NodeWrapper {
    pub fn get_index(&self) -> NodeIndex {
        self.index
    }

    pub fn set_index(&mut self, index: NodeIndex) {
        self.index = index;
    }

    pub fn get_input_connection(&self) -> Option<Connection> {
        self.input
    }

    pub fn get_output_connections(&self) -> Vec<Connection> {
        self.outputs
    }

    pub fn get_input_connection_by_type(&self, input_socket_type: SocketType) -> Option<Connection> {
        if let Some(input_socket) = self.input {
            if input_socket.to_socket_type == input_socket_type  {
                return self.input;
            }
        }

        None
    }

    pub fn get_output_connections_by_type(&self, output_socket_type: SocketType) -> Vec<Connection> {
        self.outputs
            .into_iter()
            .filter(|input| input.from_socket_type == output_socket_type)
            .collect()
    }
}

#[derive(Debug, PartialEq)]
pub struct NodeIndex {
    pub index: usize,
    pub generation: u32,
}

#[derive(Clone)]
pub struct GenerationalNode {
    pub node: Rc<RefCell<NodeWrapper>>,
    pub generation: u32,
}

pub struct Connection {
    pub from_socket_type: SocketType,
    pub other_node: NodeIndex,
    pub to_socket_type: SocketType,
}

#[derive(Debug, PartialEq)]
pub enum SocketType {
    Stream(StreamSocketType),
    Midi(MidiData),
    Value(ValueType),
    MethodCall(Vec<Parameter>),
}

#[derive(Debug, PartialEq)]
pub enum StreamSocketType {
    Audio,
    Gate,
    Detune,
    Dynamic(u64),
}

#[derive(Debug, PartialEq)]
pub enum ValueType {
    Float,
    Int,
    Boolean,
    String,
}

#[derive(Debug, PartialEq)]
pub enum Parameter {
    Float(f32),
    Int(i32),
    Boolean(bool),
    String(String),
}
