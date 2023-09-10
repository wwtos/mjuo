use std::borrow::Cow;

pub(super) use std::any::Any;
pub(super) use std::collections::HashMap;

pub(super) use resource_manager::ResourceIndex;

pub(super) use crate::errors::{NodeError, NodeOk, NodeResult, NodeWarning};
pub(super) use crate::node::{
    InitResult, Ins, Node, NodeGraphAndIo, NodeIndex, NodeInitParams, NodeIo, NodeProcessContext, NodeRow, NodeRuntime,
    NodeState, Outs, ProcessResult,
};
pub(super) use crate::{
    connection::{MidiBundle, Primitive, Socket, SocketDirection, SocketType, SocketValue},
    property::{Property, PropertyType},
};
pub(super) use sound_engine::SoundConfig;

// TODO: implement all primitive types
pub fn float(val: f32) -> Option<Primitive> {
    Some(Primitive::Float(val))
}

pub fn bool(val: bool) -> Option<Primitive> {
    Some(Primitive::Boolean(val))
}

pub fn stream_input(name: &'static str) -> NodeRow {
    NodeRow::Input(
        Socket::Simple(Cow::Borrowed(name), SocketType::Stream, 1),
        SocketValue::None,
    )
}

pub fn midi_input(name: &'static str) -> NodeRow {
    NodeRow::Input(
        Socket::Simple(Cow::Borrowed(name), SocketType::Midi, 1),
        SocketValue::None,
    )
}

pub fn value_input(name: &'static str, default: Primitive) -> NodeRow {
    NodeRow::Input(
        Socket::Simple(Cow::Borrowed(name), SocketType::Value, 1),
        SocketValue::Value(default),
    )
}

pub fn stream_output(name: &'static str) -> NodeRow {
    NodeRow::Output(Socket::Simple(Cow::Borrowed(name), SocketType::Stream, 1))
}

pub fn midi_output(name: &'static str) -> NodeRow {
    NodeRow::Output(Socket::Simple(Cow::Borrowed(name), SocketType::Midi, 1))
}

pub fn value_output(name: &'static str) -> NodeRow {
    NodeRow::Output(Socket::Simple(Cow::Borrowed(name), SocketType::Value, 1))
}

pub fn property(prop_id: &str, prop_type: PropertyType, prop_default: Property) -> NodeRow {
    NodeRow::Property(prop_id.to_string(), prop_type, prop_default)
}

pub fn multiple_choice(prop_id: &str, choices: &[&str], default_choice: &str) -> NodeRow {
    NodeRow::Property(
        prop_id.to_string(),
        PropertyType::MultipleChoice(choices.iter().map(|&choice| choice.to_string()).collect()),
        Property::MultipleChoice(default_choice.to_string()),
    )
}
