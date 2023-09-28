use rhai::{Dynamic, Scope};
use sound_engine::midi::messages::{MidiData, SystemCommonMessageData, SystemRealtimeMessageData};

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

pub fn midi_to_scope(scope: &mut Scope, midi: &MidiData) {
    let message_type = match midi {
        MidiData::NoteOff {
            channel,
            note,
            velocity,
        } => {
            scope.push("channel", Dynamic::from_int(*channel as i32));
            scope.push("note", Dynamic::from_int(*note as i32));
            scope.push("velocity", Dynamic::from_float(*velocity as f32 / 127.0));

            "note off"
        }
        MidiData::NoteOn {
            channel,
            note,
            velocity,
        } => {
            scope.push("channel", Dynamic::from_int(*channel as i32));
            scope.push("note", Dynamic::from_int(*note as i32));
            scope.push("velocity", Dynamic::from_float(*velocity as f32 / 127.0));

            "note on"
        }
        MidiData::Aftertouch {
            channel,
            note,
            pressure,
        } => {
            scope.push("channel", Dynamic::from_int(*channel as i32));
            scope.push("note", Dynamic::from_int(*note as i32));
            scope.push("pressure", Dynamic::from_float(*pressure as f32 / 127.0));

            "polyphonic aftertouch"
        }
        MidiData::ControlChange {
            channel,
            controller,
            value,
        } => {
            scope.push("channel", Dynamic::from_int(*channel as i32));
            scope.push("controller", Dynamic::from_int(*controller as i32));
            scope.push("value", Dynamic::from_float(*value as f32 / 127.0));

            "control change"
        }
        MidiData::ProgramChange { channel, patch } => {
            scope.push("channel", Dynamic::from_int(*channel as i32));
            scope.push("patch", Dynamic::from_int(*patch as i32));

            "program change"
        }
        MidiData::ChannelAftertouch { channel, pressure } => {
            scope.push("channel", Dynamic::from_int(*channel as i32));
            scope.push("pressure", Dynamic::from_float(*pressure as f32 / 127.0));

            "channel aftertouch"
        }
        MidiData::PitchBend { channel, pitch_bend } => {
            scope.push("channel", Dynamic::from_int(*channel as i32));
            scope.push("pitch_bend", Dynamic::from_float(*pitch_bend as f32 / 8192.0));

            "pitch bend"
        }
        MidiData::SystemCommonMessage { data } => match data {
            SystemCommonMessageData::SystemExclusive { id, message } => {
                scope.push("message_id", *id);
                scope.push(
                    "message",
                    Dynamic::from_array(message.into_iter().map(|x| Dynamic::from_int(*x as i32)).collect()),
                );

                "system exclusive"
            }
            SystemCommonMessageData::QuarterFrame { rate, time } => {
                scope.push("rate", Dynamic::from_int(*rate as i32));
                scope.push("hours", Dynamic::from_int(time.hours as i32));
                scope.push("minutes", Dynamic::from_int(time.minutes as i32));
                scope.push("seconds", Dynamic::from_int(time.seconds as i32));

                "quarter frame"
            }
        },
        MidiData::SystemRealtimeMessage { data } => match data {
            SystemRealtimeMessageData::TimingClock => "timing clock",
            SystemRealtimeMessageData::Start => "start",
            SystemRealtimeMessageData::Continue => "continue",
            SystemRealtimeMessageData::Stop => "stop",
            SystemRealtimeMessageData::ActiveSensing => "active sensing",
            SystemRealtimeMessageData::Reset => "reset",
        },
        MidiData::MidiNone => "",
    };

    scope.push("type", message_type);
}
