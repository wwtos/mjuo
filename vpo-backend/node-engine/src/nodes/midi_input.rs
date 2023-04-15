use smallvec::SmallVec;

use crate::nodes::prelude::*;

#[derive(Debug, Default, Clone)]
pub struct MidiInNode {
    midi_in: MidiBundle,
    has_midi_been_processed: bool,
}

impl MidiInNode {
    pub fn set_midi_output(&mut self, midi_in: MidiBundle) {
        self.midi_in = midi_in;
        self.has_midi_been_processed = false;
    }
}

impl NodeRuntime for MidiInNode {
    fn get_midi_outputs(&self, midi_out: &mut [Option<MidiBundle>]) {
        if !self.midi_in.is_empty() {
            midi_out[0] = Some(self.midi_in.clone());
        }
    }

    fn process(
        &mut self,
        state: NodeProcessState,
        streams_in: &[&[f32]],
        streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        if !self.has_midi_been_processed {
            self.has_midi_been_processed = true;
        } else if !self.midi_in.is_empty() {
            self.midi_in.clear();
        }

        NodeOk::no_warnings(())
    }
}

impl Node for MidiInNode {
    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![midi_output(register("midi"), SmallVec::new())])
    }
}
