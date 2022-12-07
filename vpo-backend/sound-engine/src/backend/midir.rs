use std::sync::mpsc;
use std::{error::Error, sync::mpsc::Receiver};

use midir::{Ignore, MidiInput, MidiInputConnection};

use crate::backend::MidiClientBackend;

pub struct MidirMidiClientBackend {
    client: MidiInputConnection<()>,
    from_midi: Receiver<Vec<u8>>,
}

impl MidirMidiClientBackend {
    pub fn new() -> Result<MidirMidiClientBackend, Box<dyn Error>> {
        let (sender, receiver) = mpsc::channel();

        let mut midi_in = MidiInput::new("Mason-Jones Unit Orchestra")?;
        midi_in.ignore(Ignore::None);

        let in_port = &midi_in.ports()[0];
        let conn_in = midi_in.connect(
            &in_port,
            "Mason-Jones Unit Orchestra",
            move |stamp, message, _| {
                sender.send(message.to_vec()).unwrap();
                println!("{}: {:?} (len = {})", stamp, message, message.len());
            },
            (),
        )?;

        Ok(MidirMidiClientBackend {
            client: conn_in,
            from_midi: receiver,
        })
    }
}

impl MidiClientBackend for MidirMidiClientBackend {
    fn read(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(self.from_midi.recv().unwrap())
    }

    fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
