use sound_engine::midi::messages::MidiData;

use crate::connection::MidiSocketType;
use crate::errors::{NodeError, NodeOk};
use crate::node::{InitResult, Node, NodeInitState, NodeRow};

#[derive(Debug, Default, Clone)]
pub struct MidiInNode {
    midi_in: Vec<MidiData>,
}

impl Node for MidiInNode {
    fn accept_midi_input(&mut self, _socket_type: &MidiSocketType, value: Vec<MidiData>) {
        self.midi_in = value;
    }

    fn get_midi_output(&self, _socket_type: &MidiSocketType) -> Vec<MidiData> {
        self.midi_in.clone()
    }

    fn init(&mut self, state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        InitResult::simple(vec![NodeRow::MidiOutput(MidiSocketType::Default, vec![])])
    }
}
