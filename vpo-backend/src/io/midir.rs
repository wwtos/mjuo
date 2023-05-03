use std::sync::mpsc::Receiver;
use std::{io::Write, sync::mpsc};

use midir::{Ignore, MidiInput, MidiInputConnection};
use node_engine::connection::MidiBundle;
use snafu::ResultExt;
use sound_engine::midi::parse::MidiParser;

use crate::errors::EngineError;

pub fn connect_midir_backend() -> Result<(Receiver<MidiBundle>, MidiInputConnection<()>), EngineError> {
    let mut parser = MidiParser::new();
    let (sender, receiver) = mpsc::channel();

    let mut midi_in = MidiInput::new("Mason-Jones Unit Orchestra").whatever_context("Failed to create midi device")?;
    midi_in.ignore(Ignore::None);

    let in_port = &midi_in.ports()[0];
    let conn_in = midi_in
        .connect(
            &in_port,
            "Mason-Jones Unit Orchestra",
            move |_stamp, message, _data| {
                parser.write(message).unwrap();

                if !parser.parsed.is_empty() {
                    let messages: MidiBundle = parser.parsed.drain(..).collect();

                    sender.send(messages).unwrap();
                }
            },
            (),
        )
        .whatever_context("Failed to connect to midi device")?;

    Ok((receiver, conn_in))
}
