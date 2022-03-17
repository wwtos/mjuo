use serde::{Deserialize, Serialize};
use sound_engine::midi::messages::MidiData;
use sound_engine::node::AudioNode;
use sound_engine::node::oscillator::Oscillator;
use sound_engine::node::oscillator::{Waveform};

use crate::connection::{SocketType, StreamSocketType, MidiSocketType};
use crate::node::Node;

#[derive(Debug, Serialize, Deserialize)]
pub struct MidiInNode {
    midi_in: Vec<MidiData>
}

impl Default for MidiInNode {
    fn default() -> Self {
        MidiInNode {
            midi_in: Vec::new()
        }
    }
}

impl Node for MidiInNode {
    fn accept_midi_input(&mut self, _socket_type: MidiSocketType, value: Vec<MidiData>) {
        self.midi_in = value;
    }

    fn get_midi_output(&self, _socket_type: MidiSocketType) -> Vec<MidiData> {
        self.midi_in.clone()
    }

    fn list_input_sockets(&self) -> Vec<SocketType> {
        vec![]
    }

    fn list_output_sockets(&self) -> Vec<SocketType> {
        vec![SocketType::Midi(MidiSocketType::Default)]
    }
}
