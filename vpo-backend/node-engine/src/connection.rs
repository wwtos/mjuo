use rhai::Dynamic;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use sound_engine::midi::messages::MidiData;

use std::fmt::{Debug, Display};

use crate::{node::NodeIndex, node_graph::NodeConnection, socket_registry::SocketRegistry};

pub type MidiBundle = SmallVec<[MidiData; 2]>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub from_node: NodeIndex,
    pub to_node: NodeIndex,
    pub data: NodeConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InputSideConnection {
    pub from_socket_type: SocketType,
    pub from_node: NodeIndex,
    pub to_socket_type: SocketType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputSideConnection {
    pub from_socket_type: SocketType,
    pub to_node: NodeIndex,
    pub to_socket_type: SocketType,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum SocketType {
    Stream(StreamSocketType),
    Midi(MidiSocketType),
    Value(ValueSocketType),
    NodeRef(NodeRefSocketType),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "variant", content = "data")]
pub enum SocketDirection {
    Input,
    Output,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, Eq)]
#[serde(tag = "variant", content = "data")]
pub enum MidiSocketType {
    Default,
    Dynamic(u64),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum StreamSocketType {
    Audio,
    Gate,
    Gain,
    Detune,
    Dynamic(u64),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum ValueSocketType {
    Default,
    Gain,
    Frequency,
    Resonance,
    Gate,
    Attack,
    Decay,
    Sustain,
    Release,
    Speed,
    State,
    UiState,
    Dynamic(u64),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum NodeRefSocketType {
    Button,
    Dynamic(u64),
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

impl SocketType {
    #[inline]
    pub fn get_name(&self) -> &str {
        match self {
            SocketType::Stream(_) => "string",
            SocketType::Midi(_) => "midi",
            SocketType::Value(_) => "value",
            SocketType::NodeRef(_) => "node_ref",
        }
    }

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

    /// This function populates a socket registry with all of the existing default socket types
    #[inline]
    pub fn register_defaults(registry: &mut SocketRegistry) {
        let socket_list = [
            ("stream.audio", SocketType::Stream(StreamSocketType::Audio)),
            ("stream.gate", SocketType::Stream(StreamSocketType::Gate)),
            ("stream.gain", SocketType::Stream(StreamSocketType::Gain)),
            ("stream.detune", SocketType::Stream(StreamSocketType::Detune)),
            ("midi.default", SocketType::Midi(MidiSocketType::Default)),
            ("value.default", SocketType::Value(ValueSocketType::Default)),
            ("value.gain", SocketType::Value(ValueSocketType::Gain)),
            ("value.frequency", SocketType::Value(ValueSocketType::Frequency)),
            ("value.resonance", SocketType::Value(ValueSocketType::Resonance)),
            ("value.gate", SocketType::Value(ValueSocketType::Gate)),
            ("value.attack", SocketType::Value(ValueSocketType::Attack)),
            ("value.decay", SocketType::Value(ValueSocketType::Decay)),
            ("value.sustain", SocketType::Value(ValueSocketType::Sustain)),
            ("value.release", SocketType::Value(ValueSocketType::Release)),
            ("value.speed", SocketType::Value(ValueSocketType::Speed)),
            ("value.state", SocketType::Value(ValueSocketType::State)),
        ];

        for socket in socket_list {
            registry
                .register_socket(socket.0.to_string(), socket.1, socket.0.to_string(), None)
                .unwrap();
        }
    }
}
