pub(super) use std::any::Any;
pub(super) use std::collections::HashMap;

pub(super) use resource_manager::ResourceIndex;

pub(super) use crate::errors::{NodeError, NodeOk, NodeResult, NodeWarning};
pub(super) use crate::node::{
    midi_input, midi_output, multiple_choice, property, stream_input, stream_output, value_input, value_output,
    InitResult, Ins, Node, NodeGraphAndIo, NodeIndex, NodeInitParams, NodeIo, NodeProcessGlobals, NodeRow, NodeRuntime,
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
