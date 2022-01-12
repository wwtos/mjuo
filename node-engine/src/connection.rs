use serde::Serialize;
use serde_json::json;

use crate::errors::{Error, ErrorType};
use crate::node::NodeIndex;

#[derive(Debug, Clone)]
pub struct Connection {
    pub from_socket_type: SocketType,
    pub from_node: NodeIndex,
    pub to_socket_type: SocketType,
    pub to_node: NodeIndex,
}

impl Connection {
    pub fn serialize_to_json(&self) -> Result<serde_json::Value, Error> {
        Ok(json!([
            self.from_socket_type.serialize_to_json()?,
            self.from_node.index,
            self.to_socket_type.serialize_to_json()?,
            self.to_node.index
        ]))
    }
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

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum SocketType {
    Stream(StreamSocketType),
    Midi(MidiSocketType),
    Value(ValueType),
    MethodCall(Vec<Parameter>),
}

impl SocketType {
    pub fn serialize_to_json(&self) -> Result<serde_json::Value, Error> {
        match serde_json::to_value(self) {
            Ok(result) => Ok(result),
            Err(error) => Err(Error::new(error.to_string(), ErrorType::ParserError)),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum MidiSocketType {
    Default,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum StreamSocketType {
    Audio,
    Gate,
    Detune,
    Dynamic(u64),
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum ValueType {
    Gain,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum Parameter {
    Float(f32),
    Int(i32),
    Boolean(bool),
    String(String),
}
