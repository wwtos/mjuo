use std::io::Write;

use midir::{Ignore, MidiInput, MidiInputConnection};
use snafu::ResultExt;
use sound_engine::midi::messages::MidiMessage;
use sound_engine::midi::parse::MidiParser;

use crate::errors::EngineError;

pub fn connect_midir_backend() -> Result<(flume::Receiver<Vec<MidiMessage>>, MidiInputConnection<()>), EngineError> {
    let mut parser = MidiParser::new();
    let (sender, receiver) = flume::unbounded();

    let mut midi_in = MidiInput::new("Mason-Jones Unit Orchestra").whatever_context("Failed to create midi device")?;
    midi_in.ignore(Ignore::None);

    let in_port = &midi_in.ports()[0];
    let conn_in = midi_in
        .connect(
            in_port,
            "Mason-Jones Unit Orchestra",
            move |stamp, message, _data| {
                parser.write_all(message).unwrap();

                if !parser.parsed.is_empty() {
                    let messages: Vec<MidiMessage> = parser
                        .parsed
                        .drain(..)
                        .map(|data| MidiMessage {
                            data,
                            timestamp: stamp as i64,
                        })
                        .collect();

                    sender.send(messages).unwrap();
                }
            },
            (),
        )
        .whatever_context("Failed to connect to midi device")?;

    Ok((receiver, conn_in))
}
