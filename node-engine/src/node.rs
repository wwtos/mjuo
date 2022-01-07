use std::fmt::Debug;
use std::{cell::RefCell, rc::Rc};

use sound_engine::midi::messages::MidiData;

pub trait Node: Debug {
    fn list_input_sockets(&self) -> Vec<SocketType>;
    fn list_output_sockets(&self) -> Vec<SocketType>;
}

#[derive(Debug)]
pub struct NodeWrapper {
    node: Box<dyn Node>,
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

    pub fn has_input_socket(&self, socket_type: &SocketType) -> bool {
        self.node.list_input_sockets().iter().find(|socket| *socket == socket_type).is_some()
    }

    pub fn has_output_socket(&self, socket_type: &SocketType) -> bool {
        self.node.list_output_sockets().iter().find(|socket| *socket == socket_type).is_some()
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

#[derive(Debug, Clone)]
pub struct Connection {
    pub from_socket_type: SocketType,
    pub from_node: NodeIndex,
    pub to_socket_type: SocketType,
    pub to_node: NodeIndex,
}

#[derive(Debug, Clone)]
pub struct InputSideConnection {
    pub from_socket_type: SocketType,
    pub from_node: NodeIndex,
    pub to_socket_type: SocketType,
}

#[derive(Debug, Clone)]
pub struct OutputSideConnection {
    pub from_socket_type: SocketType,
    pub to_node: NodeIndex,
    pub to_socket_type: SocketType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SocketType {
    Stream(StreamSocketType),
    Midi(MidiSocketType),
    Value(ValueType),
    MethodCall(Vec<Parameter>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum MidiSocketType {
    Default,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StreamSocketType {
    Audio,
    Gate,
    Detune,
    Dynamic(u64),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ValueType {
    Float,
    Int,
    Boolean,
    String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Parameter {
    Float(f32),
    Int(i32),
    Boolean(bool),
    String(String),
}
