use serde::{Deserialize, Serialize};

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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum SocketType {
    Stream(StreamSocketType),
    Midi(MidiSocketType),
    Value(ValueSocketType),
    MethodCall(Vec<Parameter>),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum MidiSocketType {
    Default,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum StreamSocketType {
    Audio,
    Gate,
    Detune,
    Dynamic(u64),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum ValueSocketType {
    Gain,
    Frequency,
    Gate,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum Parameter {
    Float(f32),
    Int(i32),
    Boolean(bool),
    String(String),
}

impl Parameter {
    #[inline]
    pub fn as_float(self) -> Option<f32> {
        match self {
            Parameter::Float(float) => Some(float),
            Parameter::Int(int) => Some(int as f32),
            Parameter::Boolean(boolean) => Some(if boolean { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    #[inline]
    pub fn as_int(self) -> Option<i32> {
        match self {
            Parameter::Int(int) => Some(int),
            Parameter::Boolean(boolean) => Some(if boolean { 1 } else { 0 }),
            _ => None,
        }
    }

    #[inline]
    pub fn as_boolean(self) -> Option<bool> {
        match self {
            Parameter::Boolean(boolean) => Some(boolean),
            _ => None,
        }
    }

    #[inline]
    pub fn as_string(self) -> Option<String> {
        match self {
            Parameter::String(string) => Some(string),
            Parameter::Float(float) => Some(float.to_string()),
            Parameter::Int(int) => Some(int.to_string()),
            Parameter::Boolean(boolean) => Some(boolean.to_string()),
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

pub fn socket_type_to_string(socket_type: SocketType) -> String {
    match socket_type {
        SocketType::Stream(stream) => match stream {
            StreamSocketType::Audio => "Audio".to_string(),
            StreamSocketType::Gate => "Gate".to_string(),
            StreamSocketType::Detune => "Detune".to_string(),
            StreamSocketType::Dynamic(_) => "Dynamic".to_string(),
        },
        SocketType::Midi(midi) => match midi {
            MidiSocketType::Default => "Midi".to_string(),
        },
        SocketType::Value(value) => match value {
            ValueSocketType::Gain => "Gain value".to_string(),
            ValueSocketType::Frequency => "Frequency value".to_string(),
            ValueSocketType::Gate => "Gate value".to_string(),
        },
        SocketType::MethodCall(method_call) => {
            format!("{:?}", method_call)
        }
    }
}
