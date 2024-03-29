use std::borrow::Cow;

pub(super) use clocked::midi::{MidiData, MidiMessage};
use common::resource_manager::ResourceId;
pub(super) use sound_engine::{MidiChannel, SoundConfig};

pub(super) use crate::errors::{NodeError, NodeOk, NodeResult, NodeWarning};
pub(super) use crate::node::{
    midi_store::MidiStore, InitResult, Ins, Node, NodeGetIoContext, NodeIndex, NodeInitParams, NodeIo,
    NodeProcessContext, NodeRow, NodeRuntime, NodeState, Outs,
};
pub(super) use crate::resources::Resource;
pub(super) use crate::{
    connection::{Primitive, Socket, SocketDirection, SocketType, SocketValue},
    property::{Property, PropertyType},
};

pub(super) use common::SeaHashMap;

// TODO: implement all primitive types
pub fn float(val: f32) -> Primitive {
    Primitive::Float(val)
}

pub fn int(val: i32) -> Primitive {
    Primitive::Int(val)
}

pub fn bool(val: bool) -> Primitive {
    Primitive::Boolean(val)
}

pub fn stream_input(name: &'static str, polyphony: usize) -> NodeRow {
    NodeRow::Input(
        Socket::Simple(Cow::Borrowed(name), SocketType::Stream, polyphony),
        SocketValue::None,
    )
}

pub fn midi_input(name: &'static str, polyphony: usize) -> NodeRow {
    NodeRow::Input(
        Socket::Simple(Cow::Borrowed(name), SocketType::Midi, polyphony),
        SocketValue::None,
    )
}

pub fn value_input(name: &'static str, default: Primitive, polyphony: usize) -> NodeRow {
    NodeRow::Input(
        Socket::Simple(Cow::Borrowed(name), SocketType::Value, polyphony),
        SocketValue::Value(default),
    )
}

pub fn stream_output(name: &'static str, polyphony: usize) -> NodeRow {
    NodeRow::Output(Socket::Simple(Cow::Borrowed(name), SocketType::Stream, polyphony))
}

pub fn midi_output(name: &'static str, polyphony: usize) -> NodeRow {
    NodeRow::Output(Socket::Simple(Cow::Borrowed(name), SocketType::Midi, polyphony))
}

pub fn value_output(name: &'static str, polyphony: usize) -> NodeRow {
    NodeRow::Output(Socket::Simple(Cow::Borrowed(name), SocketType::Value, polyphony))
}

pub fn resource(prop_id: &str, namespace: &str) -> NodeRow {
    NodeRow::Property(
        prop_id.into(),
        PropertyType::Resource(namespace.into()),
        Property::Resource(ResourceId {
            namespace: namespace.into(),
            resource: "".into(),
        }),
    )
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

pub fn with_channels(default_channel_count: usize) -> NodeRow {
    property(
        "channels",
        PropertyType::Integer,
        Property::Integer(default_channel_count.max(1) as i32),
    )
}

pub fn default_channels(props: &SeaHashMap<String, Property>, default: usize) -> usize {
    match props.get("channels") {
        Some(prop) => prop.as_integer().map(|x| x.max(1) as usize).unwrap_or(default),
        None => default,
    }
}
