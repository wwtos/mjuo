use std::mem;

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
    fn process(
        &mut self,
        globals: NodeProcessGlobals,
        ins: Ins,
        outs: Outs,
        resources: &[Option<(ResourceIndex, &dyn Any)>],
    ) -> NodeResult<()> {
        if !self.midi_in.is_empty() {
            outs.midis[0] = Some(mem::replace(&mut self.midi_in, SmallVec::new()));
        }

        ProcessResult::nothing()
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
