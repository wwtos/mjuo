use rhai::Dynamic;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use sound_engine::midi::messages::MidiData;

use std::fmt::{Debug, Display};

use crate::{node::NodeIndex, node_graph::NodeConnection};

pub type MidiBundle = SmallVec<[MidiData; 2]>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection<'a> {
    pub from_node: NodeIndex,
    pub to_node: NodeIndex,
    pub data: NodeConnection<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InputSideConnection<'a> {
    pub from_socket: Socket<'a>,
    pub from_node: NodeIndex,
    pub to_socket: Socket<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputSideConnection<'a> {
    pub from_socket: Socket<'a>,
    pub to_node: NodeIndex,
    pub to_socket: Socket<'a>,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum Socket<'a> {
    Simple(&'a str, SocketType, usize),
    Numbered(&'a str, i32, SocketType, usize),
}

impl<'a> Socket<'a> {
    pub fn socket_type(&self) -> SocketType {
        match self {
            Self::Simple(_, socket_type, _) => *socket_type,
            Self::Numbered(_, _, socket_type, _) => *socket_type,
        }
    }

    pub fn to_owned(&self) -> OwnedSocket {
        match self {
            Socket::Simple(ref name, socket_type, polyphony) => {
                OwnedSocket::Simple(name.to_string(), *socket_type, *polyphony)
            }
            Socket::Numbered(ref name, number, socket_type, polyphony) => {
                OwnedSocket::Numbered(name.to_string(), *number, *socket_type, *polyphony)
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum OwnedSocket {
    Simple(String, SocketType, usize),
    Numbered(String, i32, SocketType, usize),
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum SocketType {
    Stream,
    Midi,
    Value,
    NodeRef,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "variant", content = "data")]
pub enum SocketDirection {
    Input,
    Output,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum Primitive {
    Float(f32),
    Int(i32),
    Boolean(bool),
    String(String),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum SocketValue {
    Stream(f32),
    Midi(MidiBundle),
    Value(Primitive),
    NodeRef,
    None,
}

impl SocketValue {
    pub fn as_stream(self) -> Option<f32> {
        match self {
            SocketValue::Stream(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_midi(self) -> Option<MidiBundle> {
        match self {
            SocketValue::Midi(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_value(self) -> Option<Primitive> {
        match self {
            SocketValue::Value(value) => Some(value),
            _ => None,
        }
    }
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
            Primitive::Float(float) => Some(float as i32),
            Primitive::Int(int) => Some(int),
            Primitive::Boolean(boolean) => Some(i32::from(boolean)),
            _ => None,
        }
    }

    #[inline]
    pub fn as_boolean(self) -> Option<bool> {
        match self {
            Primitive::Float(float) => Some(float > 0.01),
            Primitive::Int(int) => Some(int > 0),
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

    #[inline]
    pub fn as_dynamic(self) -> Dynamic {
        match self {
            Primitive::String(string) => Dynamic::from(string),
            Primitive::Float(float) => Dynamic::from(float),
            Primitive::Int(int) => Dynamic::from(int),
            Primitive::Boolean(boolean) => Dynamic::from(boolean),
        }
    }
}

impl Display for SocketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Debug::fmt(&self, f)
    }
}
