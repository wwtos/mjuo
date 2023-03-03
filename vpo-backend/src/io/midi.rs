use std::collections::VecDeque;

use midir::{Ignore, MidiInput, MidiInputPort};
use snafu::ResultExt;

use node_engine::errors::NodeError;
use sound_engine::midi::messages::{MidiData, MidiMessage};

struct Channel {
    midi_in: VecDeque<MidiMessage>,
}

pub struct MidiInterface {
    channels: Vec<Channel>,
    midi_in: MidiInput,
}

impl MidiInterface {
    pub fn new() -> Result<MidiInterface, NodeError> {
        let mut midi_in = MidiInput::new("Mason-Jones Unit Orchestra").unwrap();
        midi_in.ignore(Ignore::None);

        Ok(MidiInterface {
            channels: Vec::new(),
            midi_in,
        })
    }

    pub fn list_inputs(&self) -> Vec<MidiInputPort> {
        self.midi_in.ports()
    }
}
