use serde::{Deserialize, Serialize};
use sound_engine::midi::messages::MidiData;
use strum_macros::EnumDiscriminants;

use std::fmt::{Debug, Display};

use crate::node::NodeIndex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from_socket_type: SocketType,
    pub from_node: NodeIndex,
    pub to_socket_type: SocketType,
    pub to_node: NodeIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSideConnection {
    pub from_socket_type: SocketType,
    pub from_node: NodeIndex,
    pub to_socket_type: SocketType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSideConnection {
    pub from_socket_type: SocketType,
    pub to_node: NodeIndex,
    pub to_socket_type: SocketType,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, EnumDiscriminants)]
#[serde(tag = "type", content = "content")]
pub enum SocketType {
    Stream(StreamSocketType),
    Midi(MidiSocketType),
    Value(ValueSocketType),
    NodeRef(NodeRefSocketType),
    MethodCall(Vec<Primitive>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum SocketDirection {
    Input,
    Output
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum MidiSocketType {
    Default,
    Dynamic(u64),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum StreamSocketType {
    Audio,
    Gate,
    Gain,
    Detune,
    Dynamic(u64),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum ValueSocketType {
    Gain,
    Frequency,
    Gate,
    Dynamic(u64),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum NodeRefSocketType {
    Button,
    Dynamic(u64),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum Primitive {
    Float(f32),
    Int(i32),
    Boolean(bool),
    String(String),
}

impl Primitive {
    #[inline]
    pub fn as_float(self) -> Option<f32> {
        match self {
            Primitive::Float(float) => Some(float),
            Primitive::Int(int) => Some(int as f32),
            Primitive::Boolean(boolean) => Some(if boolean { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    #[inline]
    pub fn as_int(self) -> Option<i32> {
        match self {
            Primitive::Int(int) => Some(int),
            Primitive::Boolean(boolean) => Some(if boolean { 1 } else { 0 }),
            _ => None,
        }
    }

    #[inline]
    pub fn as_boolean(self) -> Option<bool> {
        match self {
            Primitive::Boolean(boolean) => Some(boolean),
            _ => None,
        }
    }

    #[inline]
    pub fn as_string(self) -> Option<String> {
        match self {
            Primitive::String(string) => Some(string),
            Primitive::Float(float) => Some(float.to_string()),
            Primitive::Int(int) => Some(int.to_string()),
            Primitive::Boolean(boolean) => Some(boolean.to_string()),
        }
    }
}

impl Display for SocketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Debug::fmt(&self, f)
    }
}

impl SocketType {
    #[inline]
    pub fn as_stream(self) -> Option<StreamSocketType> {
        match self {
            SocketType::Stream(stream) => Some(stream),
            _ => None,
        }
    }

    #[inline]
    pub fn as_midi(self) -> Option<MidiSocketType> {
        match self {
            SocketType::Midi(midi) => Some(midi),
            _ => None,
        }
    }

    #[inline]
    pub fn as_value(self) -> Option<ValueSocketType> {
        match self {
            SocketType::Value(value) => Some(value),
            _ => None,
        }
    }
}
