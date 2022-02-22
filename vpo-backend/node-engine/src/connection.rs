use serde::{Deserialize, Serialize};

use std::fmt::{Display, Debug};

use crate::node::NodeIndex;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SocketType {
    Stream(StreamSocketType),
    Midi(MidiSocketType),
    Value(ValueType),
    MethodCall(Vec<Parameter>),
}

impl Display for SocketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Debug::fmt(&self, f)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MidiSocketType {
    Default,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamSocketType {
    Audio,
    Gate,
    Detune,
    Dynamic(u64),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ValueType {
    Gain,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Parameter {
    Float(f32),
    Int(i32),
    Boolean(bool),
    String(String),
}
