use common::{
    osc::{OscArg, OscMessageView},
    osc_midi::get_channel,
};
use rhai::{Dynamic, Scope};

use crate::connection::Primitive;

#[derive(Debug, Clone)]
pub enum ProcessState<T> {
    Unprocessed(T),
    Processed,
    None,
}

impl<T> ProcessState<T> {
    pub fn as_unprocessed(self) -> Option<T> {
        match self {
            ProcessState::Unprocessed(value) => Some(value),
            _ => None,
        }
    }
}

pub fn value_to_dynamic(value: serde_json::Value) -> Dynamic {
    match value {
        serde_json::Value::Null => Dynamic::from(()),
        serde_json::Value::Bool(value) => Dynamic::from(value),
        serde_json::Value::Number(value) => {
            if value.is_i64() {
                Dynamic::from(value.as_i64().unwrap() as i32)
            } else {
                Dynamic::from(value.as_f64().unwrap() as f32)
            }
        }
        serde_json::Value::String(value) => Dynamic::from(value),
        serde_json::Value::Array(array) => Dynamic::from(array.into_iter().map(value_to_dynamic)),
        serde_json::Value::Object(object) => Dynamic::from(
            object
                .into_iter()
                .map(|(k, v)| (smartstring::SmartString::from(k), value_to_dynamic(v)))
                .collect::<rhai::Map>(),
        ),
    }
}

pub fn dynamic_to_primitive(dynamic: Dynamic) -> Primitive {
    match dynamic.type_name() {
        "bool" => Primitive::Boolean(dynamic.as_bool().unwrap()),
        "i32" => Primitive::Int(dynamic.as_int().unwrap()),
        "f32" => Primitive::Float(dynamic.as_float().unwrap()),
        "()" => Primitive::None,
        _ => Primitive::None,
    }
}

pub fn add_message_to_scope(scope: &mut Scope, osc: &OscMessageView) {
    let address = String::from_utf8(osc.address().to_bytes().to_owned());

    if let Ok(address) = address {
        scope.push("address", address.clone());
    }

    if let Some(channel) = get_channel(osc) {
        scope.push("channel", channel as i32);
    }

    let mut args: Vec<Dynamic> = Vec::with_capacity(osc.type_tag().to_bytes().len() - 1);

    for arg in osc.arg_iter() {
        args.push(match arg {
            OscArg::True => Dynamic::from(true),
            OscArg::False => Dynamic::from(false),
            OscArg::Impulse => Dynamic::from("impulse"),
            OscArg::Blob(blob) => Dynamic::from(Vec::from(blob)),
            OscArg::String(string) => Dynamic::from(String::from_utf8(string.to_bytes().to_owned())),
            OscArg::Null => Dynamic::from(()),
            OscArg::Float(num) => Dynamic::from(num),
            OscArg::Integer(num) => Dynamic::from(num),
            OscArg::Timetag(tag) => Dynamic::from(tag.clone()),
        })
    }
}
