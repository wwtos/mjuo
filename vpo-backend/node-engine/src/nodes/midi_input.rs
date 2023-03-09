use smallvec::SmallVec;

use crate::connection::{MidiBundle, MidiSocketType};
use crate::errors::{NodeError, NodeOk};
use crate::node::{InitResult, Node, NodeInitState, NodeRow};

#[derive(Debug, Default, Clone)]
pub struct MidiInNode {
    midi_in: MidiBundle,
}

impl Node for MidiInNode {
    fn accept_midi_inputs(&mut self, midi_in: &[Option<MidiBundle>]) {
        self.midi_in = midi_in[0].unwrap();
    }

    fn get_midi_outputs(&self, midi_out: &mut [Option<MidiBundle>]) {
        if !self.midi_in.is_empty() {
            midi_out[0] = Some(self.midi_in.clone());
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
