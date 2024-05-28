use common::osc::{BundleWriter, OscTime};
use common::resource_manager::ResourceId;
use std::borrow::Cow;
use std::io::{Cursor, Write};

pub(super) use common::osc::OscView;
pub(super) use common::osc_midi::{write_message, write_note_off, write_note_on, NOTE_OFF, NOTE_ON, PITCH_BEND};
pub(super) use common::read_osc;
pub(super) use common::SeaHashMap;
pub(super) use sound_engine::SoundConfig;

pub(super) use crate::errors::{NodeError, NodeOk, NodeResult, NodeWarning};
pub(super) use crate::node::OptionExt;
use crate::node::OscIndex;
pub(super) use crate::node::{
    osc_store::OscStore, InitResult, Ins, Node, NodeGetIoContext, NodeIndex, NodeInitParams, NodeIo,
    NodeProcessContext, NodeRow, NodeRuntime, NodeState, Outs,
};
pub(super) use crate::resources::Resource;
pub(super) use crate::{
    connection::{Primitive, Socket, SocketDirection, SocketType, SocketValue},
    property::{Property, PropertyType},
};

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

pub fn write_bundle_and_message_scratch(store: &mut OscStore, scratch: &Vec<u8>) -> Option<OscIndex> {
    if !scratch.is_empty() {
        let total_len = 8 + 8 + scratch.len();

        store.add_osc(total_len, |bytes| {
            let mut cursor = Cursor::new(bytes);

            // use bundle writer to write the header
            BundleWriter::start(Some(&mut cursor), OscTime::default()).unwrap();
            cursor.write_all(&scratch[..]).unwrap();
        })
    } else {
        None
    }
}

#[inline]
pub(super) fn default_osc() -> Vec<u8> {
    Vec::with_capacity(512)
}

pub trait HashMapExt {
    fn get_string(&self, k: &str) -> Result<String, NodeError>;

    fn get_bool(&self, k: &str) -> Result<bool, NodeError>;

    fn get_int(&self, k: &str) -> Result<i32, NodeError>;

    fn get_float(&self, k: &str) -> Result<f32, NodeError>;

    fn get_resource(&self, k: &str) -> Result<ResourceId, NodeError>;

    fn get_multiple_choice(&self, k: &str) -> Result<String, NodeError>;
}

impl HashMapExt for SeaHashMap<String, Property> {
    fn get_string(&self, k: &str) -> Result<String, NodeError> {
        self.get(k)
            .cloned()
            .ok_or(NodeError::MissingProperty {
                property: k.to_string(),
            })?
            .as_string()
            .ok_or(NodeError::WrongPropertyType {
                property: k.to_string(),
            })
    }

    fn get_bool(&self, k: &str) -> Result<bool, NodeError> {
        self.get(k)
            .ok_or(NodeError::MissingProperty {
                property: k.to_string(),
            })?
            .as_bool()
            .ok_or(NodeError::WrongPropertyType {
                property: k.to_string(),
            })
    }

    fn get_int(&self, k: &str) -> Result<i32, NodeError> {
        self.get(k)
            .ok_or(NodeError::MissingProperty {
                property: k.to_string(),
            })?
            .as_integer()
            .ok_or(NodeError::WrongPropertyType {
                property: k.to_string(),
            })
    }

    fn get_float(&self, k: &str) -> Result<f32, NodeError> {
        self.get(k)
            .ok_or(NodeError::MissingProperty {
                property: k.to_string(),
            })?
            .as_float()
            .ok_or(NodeError::WrongPropertyType {
                property: k.to_string(),
            })
    }

    fn get_resource(&self, k: &str) -> Result<ResourceId, NodeError> {
        self.get(k)
            .cloned()
            .ok_or(NodeError::MissingProperty {
                property: k.to_string(),
            })?
            .as_resource()
            .ok_or(NodeError::WrongPropertyType {
                property: k.to_string(),
            })
    }

    fn get_multiple_choice(&self, k: &str) -> Result<String, NodeError> {
        self.get(k)
            .cloned()
            .ok_or(NodeError::MissingProperty {
                property: k.to_string(),
            })?
            .as_multiple_choice()
            .ok_or(NodeError::WrongPropertyType {
                property: k.to_string(),
            })
    }
}
