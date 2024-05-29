use std::{ffi::CStr, io::Write};

use cstr::cstr;
use paste::paste;

use clocked::midi::{MidiData, SysCommon, SysRt, Timecode};

use crate::osc::{write_osc_message, write_osc_message_prepend_len, NoopWriter, OscArg, OscMessageView};

macro_rules! osc_const {
    ( $( ( $name:ident, $value:literal ) ),* ) => {
        paste! {
            $(
                pub const $name: &'static str = $value;
                pub const [<$name _C>]: &'static CStr = cstr!($value);

            )*
        }
    };
}

#[macro_export]
macro_rules! read_osc {
    ( $arg_iter:expr, $( $getter_func:ident ),+ ) => {
        (|| {
            let mut iter = $arg_iter;

            Some(
                ($(iter.next()?.$getter_func()?),+)
            )
        })()
    };
}

osc_const![
    (NOTE_ON, "/note_on"),
    (NOTE_OFF, "/note_off"),
    (AFTERTOUCH, "/aftertouch"),
    (CONTROL_CHANGE, "/control_change"),
    (PROGRAM_CHANGE, "/program_change"),
    (CHANNEL_PRESSURE, "/channel_pressure"),
    (PITCH_BEND, "/pitch_bend"),
    (COMMON_QUARTER_FRAME_FRAME_LOW, "/common/quarter_frame/frame_low"),
    (COMMON_QUARTER_FRAME_FRAME_HIGH, "/common/quarter_frame/frame_high"),
    (COMMON_QUARTER_FRAME_SECONDS_LOW, "/common/quarter_frame/seconds_low"),
    (COMMON_QUARTER_FRAME_SECONDS_HIGH, "/common/quarter_frame/seconds_high"),
    (COMMON_QUARTER_FRAME_MINUTES_LOW, "/common/quarter_frame/minutes_low"),
    (COMMON_QUARTER_FRAME_MINUTES_HIGH, "/common/quarter_frame/minutes_high"),
    (COMMON_QUARTER_FRAME_HOURS_LOW, "/common/quarter_frame/hours_low"),
    (COMMON_QUARTER_FRAME_HOURS_HIGH, "/common/quarter_frame/hours_high"),
    (COMMON_SONG_POSITION_POINTER, "/common/song_position_pointer"),
    (COMMON_SONG_SELECT, "/common/song_select"),
    (COMMON_TUNE_REQUEST, "/common/tune_request"),
    (REALTIME_MIDI_CLOCK, "/realtime/midi_clock"),
    (REALTIME_TICK, "/realtime/tick"),
    (REALTIME_START, "/realtime/start"),
    (REALTIME_CONTINUE, "/realtime/continue"),
    (REALTIME_STOP, "/realtime/stop"),
    (REALTIME_ACTIVE_SENSING, "/realtime/active_sensing"),
    (REALTIME_RESET, "/realtime/reset"),
    (SYSTEM_EXCLUSIVE, "/system_exclusive")
];

pub fn read_osc_to_midi(message: &OscMessageView) -> Option<MidiData> {
    let address = message.address();

    if address == NOTE_ON_C {
        if let Some((channel, note, velocity)) = read_osc!(message.arg_iter(), as_int, as_int, as_int) {
            return Some(MidiData::NoteOn {
                channel: channel as u8,
                note: note as u8,
                velocity: velocity as u8,
            });
        }
    } else if address == NOTE_OFF_C {
        if let Some((channel, note, velocity)) = read_osc!(message.arg_iter(), as_int, as_int, as_int) {
            return Some(MidiData::NoteOff {
                channel: channel as u8,
                note: note as u8,
                velocity: velocity as u8,
            });
        }
    } else if address == AFTERTOUCH_C {
        if let Some((channel, note, pressure)) = read_osc!(message.arg_iter(), as_int, as_int, as_int) {
            return Some(MidiData::Aftertouch {
                channel: channel as u8,
                note: note as u8,
                pressure: pressure as u8,
            });
        }
    } else if address == CONTROL_CHANGE_C {
        if let Some((channel, controller, value)) = read_osc!(message.arg_iter(), as_int, as_int, as_int) {
            return Some(MidiData::ControlChange {
                channel: channel as u8,
                controller: controller as u8,
                value: value as u8,
            });
        }
    } else if address == PROGRAM_CHANGE_C {
        if let Some((channel, patch)) = read_osc!(message.arg_iter(), as_int, as_int) {
            return Some(MidiData::ProgramChange {
                channel: channel as u8,
                patch: patch as u8,
            });
        }
    } else if address == CHANNEL_PRESSURE_C {
        if let Some((channel, pressure)) = read_osc!(message.arg_iter(), as_int, as_int) {
            return Some(MidiData::ChannelPressure {
                channel: channel as u8,
                pressure: pressure as u8,
            });
        }
    } else if address == PITCH_BEND_C {
        if let Some((channel, pitch_bend)) = read_osc!(message.arg_iter(), as_int, as_int) {
            return Some(MidiData::PitchBend {
                channel: channel as u8,
                pitch_bend: pitch_bend as u16,
            });
        }
    } else if address == COMMON_QUARTER_FRAME_FRAME_LOW_C {
        if let Some(frame) = read_osc!(message.arg_iter(), as_int) {
            return Some(MidiData::SysCommon(SysCommon::QuarterFrame {
                time_fragment: Timecode::FrameLow(frame as u8),
            }));
        }
    } else if address == COMMON_QUARTER_FRAME_FRAME_HIGH_C {
        if let Some(frame) = read_osc!(message.arg_iter(), as_int) {
            return Some(MidiData::SysCommon(SysCommon::QuarterFrame {
                time_fragment: Timecode::FrameHigh(frame as u8),
            }));
        }
    } else if address == COMMON_QUARTER_FRAME_SECONDS_LOW_C {
        if let Some(seconds) = read_osc!(message.arg_iter(), as_int) {
            return Some(MidiData::SysCommon(SysCommon::QuarterFrame {
                time_fragment: Timecode::SecondsLow(seconds as u8),
            }));
        }
    } else if address == COMMON_QUARTER_FRAME_SECONDS_HIGH_C {
        if let Some(seconds) = read_osc!(message.arg_iter(), as_int) {
            return Some(MidiData::SysCommon(SysCommon::QuarterFrame {
                time_fragment: Timecode::SecondsHigh(seconds as u8),
            }));
        }
    } else if address == COMMON_QUARTER_FRAME_MINUTES_LOW_C {
        if let Some(minutes) = read_osc!(message.arg_iter(), as_int) {
            return Some(MidiData::SysCommon(SysCommon::QuarterFrame {
                time_fragment: Timecode::MinutesLow(minutes as u8),
            }));
        }
    } else if address == COMMON_QUARTER_FRAME_MINUTES_HIGH_C {
        if let Some(minutes) = read_osc!(message.arg_iter(), as_int) {
            return Some(MidiData::SysCommon(SysCommon::QuarterFrame {
                time_fragment: Timecode::MinutesHigh(minutes as u8),
            }));
        }
    } else if address == COMMON_QUARTER_FRAME_HOURS_LOW_C {
        if let Some(hours) = read_osc!(message.arg_iter(), as_int) {
            return Some(MidiData::SysCommon(SysCommon::QuarterFrame {
                time_fragment: Timecode::HoursLow(hours as u8),
            }));
        }
    } else if address == COMMON_QUARTER_FRAME_HOURS_HIGH_C {
        if let Some(hours) = read_osc!(message.arg_iter(), as_int) {
            return Some(MidiData::SysCommon(SysCommon::QuarterFrame {
                time_fragment: Timecode::HoursHigh(hours as u8),
            }));
        }
    } else if address == COMMON_SONG_POSITION_POINTER_C {
        if let Some(position) = read_osc!(message.arg_iter(), as_int) {
            return Some(MidiData::SysCommon(SysCommon::SongPositionPointer {
                position: position as u16,
            }));
        }
    } else if address == COMMON_SONG_SELECT_C {
        if let Some(song) = read_osc!(message.arg_iter(), as_int) {
            return Some(MidiData::SysCommon(SysCommon::SongSelect { song: song as u8 }));
        }
    } else if address == COMMON_TUNE_REQUEST_C {
        return Some(MidiData::SysCommon(SysCommon::TuneRequest));
    } else if address == REALTIME_MIDI_CLOCK_C {
        return Some(MidiData::SysRt(SysRt::MidiClock));
    } else if address == REALTIME_TICK_C {
        return Some(MidiData::SysRt(SysRt::Tick));
    } else if address == REALTIME_START_C {
        return Some(MidiData::SysRt(SysRt::Start));
    } else if address == REALTIME_CONTINUE_C {
        return Some(MidiData::SysRt(SysRt::Continue));
    } else if address == REALTIME_STOP_C {
        return Some(MidiData::SysRt(SysRt::Stop));
    } else if address == REALTIME_ACTIVE_SENSING_C {
        return Some(MidiData::SysRt(SysRt::ActiveSensing));
    } else if address == REALTIME_RESET_C {
        return Some(MidiData::SysRt(SysRt::Reset));
    } else if address == SYSTEM_EXCLUSIVE_C {
        if let Some(OscArg::Blob(data)) = message.arg_iter().next() {
            return Some(MidiData::SysEx {
                id_and_data: data.to_vec(),
            });
        }
    }

    None
}

pub fn get_channel(msg: &OscMessageView) -> Option<u8> {
    let addr = msg.address();

    if addr == NOTE_ON_C
        || addr == NOTE_OFF_C
        || addr == AFTERTOUCH_C
        || addr == CONTROL_CHANGE_C
        || addr == PROGRAM_CHANGE_C
        || addr == CHANNEL_PRESSURE_C
        || addr == PITCH_BEND_C
    {
        if let Some(channel) = read_osc!(msg.arg_iter(), as_int) {
            if channel >= 0 && channel <= 15 {
                return Some(channel as u8);
            }
        }
    }

    None
}

pub fn write_midi_as_osc<W: Write>(writer: &mut W, message: &MidiData) -> Result<usize, std::io::Error> {
    match message {
        MidiData::NoteOff {
            channel,
            note,
            velocity,
        } => write_osc_message(
            writer,
            NOTE_OFF_C,
            &[
                OscArg::Integer(*channel as i32),
                OscArg::Integer(*note as i32),
                OscArg::Integer(*velocity as i32),
            ],
        ),
        MidiData::NoteOn {
            channel,
            note,
            velocity,
        } => write_osc_message(
            writer,
            NOTE_ON_C,
            &[
                OscArg::Integer(*channel as i32),
                OscArg::Integer(*note as i32),
                OscArg::Integer(*velocity as i32),
            ],
        ),
        MidiData::Aftertouch {
            channel,
            note,
            pressure,
        } => write_osc_message(
            writer,
            AFTERTOUCH_C,
            &[
                OscArg::Integer(*channel as i32),
                OscArg::Integer(*note as i32),
                OscArg::Integer(*pressure as i32),
            ],
        ),
        MidiData::ControlChange {
            channel,
            controller,
            value,
        } => write_osc_message(
            writer,
            CONTROL_CHANGE_C,
            &[
                OscArg::Integer(*channel as i32),
                OscArg::Integer(*controller as i32),
                OscArg::Integer(*value as i32),
            ],
        ),
        MidiData::ProgramChange { channel, patch } => write_osc_message(
            writer,
            PROGRAM_CHANGE_C,
            &[OscArg::Integer(*channel as i32), OscArg::Integer(*patch as i32)],
        ),
        MidiData::ChannelPressure { channel, pressure } => write_osc_message(
            writer,
            CHANNEL_PRESSURE_C,
            &[OscArg::Integer(*channel as i32), OscArg::Integer(*pressure as i32)],
        ),
        MidiData::PitchBend { channel, pitch_bend } => write_osc_message(
            writer,
            PITCH_BEND_C,
            &[OscArg::Integer(*channel as i32), OscArg::Integer(*pitch_bend as i32)],
        ),
        MidiData::SysCommon(common) => match common {
            clocked::midi::SysCommon::QuarterFrame { time_fragment } => match time_fragment {
                clocked::midi::Timecode::FrameLow(n) => {
                    write_osc_message(writer, COMMON_QUARTER_FRAME_FRAME_LOW_C, &[OscArg::Integer(*n as i32)])
                }
                clocked::midi::Timecode::FrameHigh(n) => {
                    write_osc_message(writer, COMMON_QUARTER_FRAME_FRAME_HIGH_C, &[OscArg::Integer(*n as i32)])
                }
                clocked::midi::Timecode::SecondsLow(n) => write_osc_message(
                    writer,
                    COMMON_QUARTER_FRAME_SECONDS_LOW_C,
                    &[OscArg::Integer(*n as i32)],
                ),
                clocked::midi::Timecode::SecondsHigh(n) => write_osc_message(
                    writer,
                    COMMON_QUARTER_FRAME_SECONDS_HIGH_C,
                    &[OscArg::Integer(*n as i32)],
                ),
                clocked::midi::Timecode::MinutesLow(n) => write_osc_message(
                    writer,
                    COMMON_QUARTER_FRAME_MINUTES_LOW_C,
                    &[OscArg::Integer(*n as i32)],
                ),
                clocked::midi::Timecode::MinutesHigh(n) => write_osc_message(
                    writer,
                    COMMON_QUARTER_FRAME_MINUTES_HIGH_C,
                    &[OscArg::Integer(*n as i32)],
                ),
                clocked::midi::Timecode::HoursLow(n) => {
                    write_osc_message(writer, COMMON_QUARTER_FRAME_HOURS_LOW_C, &[OscArg::Integer(*n as i32)])
                }
                clocked::midi::Timecode::HoursHigh(n) => {
                    write_osc_message(writer, COMMON_QUARTER_FRAME_HOURS_HIGH_C, &[OscArg::Integer(*n as i32)])
                }
            },
            clocked::midi::SysCommon::SongPositionPointer { position } => write_osc_message(
                writer,
                COMMON_SONG_POSITION_POINTER_C,
                &[OscArg::Integer(*position as i32)],
            ),
            clocked::midi::SysCommon::SongSelect { song } => {
                write_osc_message(writer, COMMON_SONG_SELECT_C, &[OscArg::Integer(*song as i32)])
            }
            clocked::midi::SysCommon::TuneRequest => write_osc_message(writer, COMMON_TUNE_REQUEST_C, &[]),
        },
        MidiData::SysRt(realtime) => match realtime {
            clocked::midi::SysRt::MidiClock => write_osc_message(writer, REALTIME_MIDI_CLOCK_C, &[]),
            clocked::midi::SysRt::Tick => write_osc_message(writer, REALTIME_TICK_C, &[]),
            clocked::midi::SysRt::Start => write_osc_message(writer, REALTIME_START_C, &[]),
            clocked::midi::SysRt::Continue => write_osc_message(writer, REALTIME_CONTINUE_C, &[]),
            clocked::midi::SysRt::Stop => write_osc_message(writer, REALTIME_STOP_C, &[]),
            clocked::midi::SysRt::ActiveSensing => write_osc_message(writer, REALTIME_ACTIVE_SENSING_C, &[]),
            clocked::midi::SysRt::Reset => write_osc_message(writer, REALTIME_RESET_C, &[]),
        },
        MidiData::SysEx { id_and_data } => write_osc_message(
            writer,
            SYSTEM_EXCLUSIVE_C,
            &[OscArg::Blob(id_and_data.as_slice().into())],
        ),
        MidiData::MidiNone => Ok(0),
    }
}

pub fn write_midi_as_osc_prepend_len<W: Write>(writer: &mut W, message: &MidiData) -> Result<usize, std::io::Error> {
    let len = write_midi_as_osc(&mut NoopWriter {}, message)?;

    writer.write_all(&(len as u32).to_be_bytes())?;

    write_midi_as_osc(writer, message)
}

pub fn is_message_reset(message: &OscMessageView) -> bool {
    let addr = message.address();

    if addr == REALTIME_RESET_C {
        return true;
    } else if addr == CONTROL_CHANGE_C {
        if let Some((_channel, controller, value)) = read_osc!(message.arg_iter(), as_int, as_int, as_int) {
            if controller == 120 || controller == 121 || (controller == 122 && value == 0) || controller == 123 {
                return true;
            }
        }
    }

    false
}

/// Writes the message (and prepends message length) to the vec
pub fn write_message(vec: &mut Vec<u8>, message: &OscMessageView) {
    vec.extend_from_slice(&(message.bytes().len() as u32).to_be_bytes());
    vec.extend_from_slice(message.bytes());
}

/// Writes the note on osc message, including four bytes before
pub fn write_note_on(vec: &mut Vec<u8>, channel: u8, note: u8, velocity: u8) {
    write_osc_message_prepend_len(
        vec,
        NOTE_ON_C,
        &[
            OscArg::Integer(channel as i32),  // channel
            OscArg::Integer(note as i32),     // note
            OscArg::Integer(velocity as i32), // velocity
        ],
    )
    .unwrap();
}

/// Writes the note off osc message, including four bytes before
pub fn write_note_off(vec: &mut Vec<u8>, channel: u8, note: u8, velocity: u8) {
    write_osc_message_prepend_len(
        vec,
        NOTE_OFF_C,
        &[
            OscArg::Integer(channel as i32),  // channel
            OscArg::Integer(note as i32),     // note
            OscArg::Integer(velocity as i32), // velocity
        ],
    )
    .unwrap();
}
