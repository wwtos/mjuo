use rhai::plugin::*;
use rhai::{Dynamic, Engine, EvalAltResult};

use serde::{Deserialize, Serialize};

pub type Channel = u8;
pub type Note = u8;
pub type Velocity = u8;
pub type Pressue = u8;
pub type ControlIndex = u8;
pub type ControlValue = u8;
pub type Patch = u8;
pub type Bend = i16;
pub type ExclusiveMessage = Vec<u8>;
pub type ManufacturerID = [u8; 3];
pub type TimecodeRate = u8;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timecode {
    hours: u8,
    minutes: u8,
    seconds: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum SystemCommonMessageData {
    SystemExclusive {
        id: ManufacturerID,
        message: ExclusiveMessage,
    },
    QuarterFrame {
        rate: TimecodeRate,
        time: Timecode,
    }, // Song Position Pointer
       // Song Select
       // Tune Request
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum SystemRealtimeMessageData {
    TimingClock,
    Start,
    Continue,
    Stop,
    ActiveSensing,
    Reset,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "variant", content = "data")]
pub enum MidiData {
    NoteOff {
        channel: Channel,
        note: Note,
        velocity: Velocity,
    },
    NoteOn {
        channel: Channel,
        note: Note,
        velocity: Velocity,
    },
    Aftertouch {
        channel: Channel,
        note: Note,
        pressure: Pressue,
    },
    ControlChange {
        channel: Channel,
        controller: ControlIndex,
        value: ControlValue,
    },
    ProgramChange {
        channel: Channel,
        patch: Patch,
    },
    ChannelAftertouch {
        channel: Channel,
        pressure: Pressue,
    },
    PitchBend {
        channel: Channel,
        pitch_bend: Bend,
    },
    SystemCommonMessage {
        data: SystemCommonMessageData,
    },
    SystemRealtimeMessage {
        data: SystemRealtimeMessageData,
    },
    MidiNone,
}

pub struct MidiMessage {
    pub data: MidiData,
    pub timestamp: u64,
}
