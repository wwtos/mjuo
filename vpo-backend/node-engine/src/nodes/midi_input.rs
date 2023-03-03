use smallvec::SmallVec;

use crate::connection::{MidiBundle, MidiSocketType};
use crate::errors::{NodeError, NodeOk};
use crate::node::{InitResult, Node, NodeInitState, NodeRow};

#[derive(Debug, Default, Clone)]
pub struct MidiInNode {
    midi_in: MidiBundle,
}

impl Node for MidiInNode {
    fn accept_midi_input(&mut self, _socket_type: MidiSocketType, value: MidiBundle) {
        self.midi_in = value;
    }

    fn get_midi_output(&self, _socket_type: MidiSocketType) -> Option<MidiBundle> {
        if !self.midi_in.is_empty() {
            Some(self.midi_in.clone())
        } else {
            None
        }
    }

    fn init(&mut self, _state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        InitResult::simple(vec![NodeRow::MidiOutput(
            MidiSocketType::Default,
            SmallVec::new(),
            false,
        )])
    }
}
