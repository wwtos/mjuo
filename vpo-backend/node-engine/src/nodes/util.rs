use std::str::FromStr;

use clocked::midi::{MidiData, SysCommon, SysRt, Timecode};
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

pub fn add_message_to_scope(scope: &mut Scope, midi: &MidiData) {
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
        MidiData::ChannelPressure { channel, pressure } => {
            scope.push("channel", Dynamic::from_int(*channel as i32));
            scope.push("pressure", Dynamic::from_float(*pressure as f32 / 127.0));

            "channel aftertouch"
        }
        MidiData::PitchBend { channel, pitch_bend } => {
            scope.push("channel", Dynamic::from_int(*channel as i32));
            scope.push("pitch_bend", Dynamic::from_float(*pitch_bend as f32 / 8192.0));

            "pitch bend"
        }
        MidiData::SysEx { id_and_data } => {
            scope.push("message_id", id_and_data[0]);
            scope.push(
                "message",
                Dynamic::from_array(
                    id_and_data[1..]
                        .into_iter()
                        .map(|x| Dynamic::from_int(*x as i32))
                        .collect(),
                ),
            );

            "system exclusive"
        }
        MidiData::SysCommon(SysCommon::QuarterFrame { time_fragment }) => {
            let (time_fragment_type, time_fragment) = match time_fragment {
                Timecode::FrameLow(nibble) => ("frame low", nibble),
                Timecode::FrameHigh(nibble) => ("frame high", nibble),
                Timecode::SecondsLow(nibble) => ("seconds low", nibble),
                Timecode::SecondsHigh(nibble) => ("seconds high", nibble),
                Timecode::MinutesLow(nibble) => ("minutes low", nibble),
                Timecode::MinutesHigh(nibble) => ("minutes high", nibble),
                Timecode::HoursLow(nibble) => ("hours low", nibble),
                Timecode::HoursHigh(nibble) => ("hours high", nibble),
            };

            scope.push("time_fragment_type", Dynamic::from_str(time_fragment_type));
            scope.push("time_fragment", Dynamic::from_int(*time_fragment as i32));

            "quarter frame"
        }
        MidiData::SysCommon(SysCommon::SongPositionPointer { position }) => {
            scope.push("position", Dynamic::from_int(*position as i32));

            "song position"
        }
        MidiData::SysCommon(SysCommon::SongSelect { song }) => {
            scope.push("song", Dynamic::from_int(*song as i32));

            "song select"
        }
        MidiData::SysCommon(SysCommon::TuneRequest) => "tune request",
        MidiData::SysRt(message) => match message {
            SysRt::MidiClock => "midi clock",
            SysRt::Tick => "tick",
            SysRt::Start => "start",
            SysRt::Continue => "continue",
            SysRt::Stop => "stop",
            SysRt::ActiveSensing => "active sensing",
            SysRt::Reset => "reset",
        },
        MidiData::MidiNone => "",
    };

    scope.push("type", message_type);
}
