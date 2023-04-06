use smallvec::SmallVec;

use crate::nodes::prelude::*;

#[derive(Debug, Default, Clone)]
pub struct MidiInNode {
    midi_in: MidiBundle,
}

impl MidiInNode {
    pub fn set_midi_output(&mut self, midi_in: MidiBundle) {
        self.midi_in = midi_in;
    }
}

impl NodeRuntime for MidiInNode {
    fn get_midi_outputs(&self, midi_out: &mut [Option<MidiBundle>]) {
        if !self.midi_in.is_empty() {
            midi_out[0] = Some(self.midi_in.clone());
        }
    }
}

impl Node for MidiInNode {
    fn get_io(props: HashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![midi_output("midi", SmallVec::new())])
    }
}
