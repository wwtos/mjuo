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
    fn get_midi_outputs(&mut self, midi_out: &mut [Option<MidiBundle>]) {
        if !self.midi_in.is_empty() {
            midi_out[0] = Some(self.midi_in.clone());
        }
    }

    fn finish(&mut self) {
        self.midi_in.clear();
    }
}

impl Node for MidiInNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        MidiInNode {
            midi_in: SmallVec::new(),
        }
    }

    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![midi_output(register("midi"))])
    }
}
