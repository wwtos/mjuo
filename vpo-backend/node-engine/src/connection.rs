use rhai::Dynamic;
use serde::{Deserialize, Serialize};
use sound_engine::midi::messages::MidiMessage;

use std::{
    borrow::Cow,
    fmt::{Debug, Display},
};

use crate::{node::NodeIndex, node_graph::NodeConnectionData};

pub type MidiBundle<'bump> = bumpalo::collections::Vec<'bump, MidiMessage>;
pub type OwnedMidiBundle = Vec<MidiMessage>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub from_node: NodeIndex,
    pub to_node: NodeIndex,
    pub data: NodeConnectionData,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InputSideConnection {
    pub from_socket: Socket,
    pub from_node: NodeIndex,
    pub to_socket: Socket,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputSideConnection {
    pub from_socket: Socket,
    pub to_node: NodeIndex,
    pub to_socket: Socket,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum Socket {
    Simple(Cow<'static, str>, SocketType, usize),
    WithData(Cow<'static, str>, String, SocketType, usize),
}

impl Socket {
    pub fn socket_type(&self) -> SocketType {
        match self {
            Self::Simple(_, socket_type, _) => *socket_type,
            Self::WithData(_, _, socket_type, _) => *socket_type,
        }
    }

    pub fn channels(&self) -> usize {
        match self {
            Self::Simple(_, _, channels) => *channels,
            Self::WithData(_, _, _, channels) => *channels,
        }
    }
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

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum Primitive {
    Float(f32),
    Int(i32),
    Boolean(bool),
    None,
}

impl Primitive {
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Primitive::None)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum SocketValue {
    Stream(f32),
    Midi(OwnedMidiBundle),
    Value(Primitive),
    None,
}

impl SocketValue {
    pub fn as_stream(self) -> Option<f32> {
        match self {
            SocketValue::Stream(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_midi(self) -> Option<OwnedMidiBundle> {
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
    pub fn as_float(&self) -> Option<f32> {
        match self {
            Primitive::Float(float) => Some(*float),
            Primitive::Int(int) => Some(*int as f32),
            Primitive::Boolean(boolean) => Some(if *boolean { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    #[inline]
    pub fn as_int(&self) -> Option<i32> {
        match self {
            Primitive::Float(float) => Some(*float as i32),
            Primitive::Int(int) => Some(*int),
            Primitive::Boolean(boolean) => Some(i32::from(*boolean)),
            _ => None,
        }
    }

    #[inline]
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Primitive::Float(float) => Some(*float > 0.01),
            Primitive::Int(int) => Some(*int > 0),
            Primitive::Boolean(boolean) => Some(*boolean),
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
            Primitive::None => None,
        }
    }

    #[inline]
    pub fn as_dynamic(self) -> Dynamic {
        match self {
            Primitive::String(string) => Dynamic::from(string),
            Primitive::Float(float) => Dynamic::from(float),
            Primitive::Int(int) => Dynamic::from(int),
            Primitive::Boolean(boolean) => Dynamic::from(boolean),
            Primitive::None => Dynamic::from(()),
        }
    }
}

impl Display for SocketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Debug::fmt(&self, f)
    }
}
