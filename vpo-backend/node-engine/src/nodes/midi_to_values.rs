use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct MidiToValuesNode {}

impl NodeRuntime for MidiToValuesNode {
    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        midi_store: &mut MidiStore,
        _resources: &[Resource],
    ) -> NodeResult<()> {
        if let Some(midi) = &ins.midi(0)[0] {
            let messages = midi_store.borrow_midi(midi).unwrap();

            for data in messages.iter() {
                match &data.data {
                    MidiData::NoteOn {
                        channel: _,
                        note,
                        velocity,
                    } => {
                        outs.value(0)[0] = float(440.0 * f32::powf(2.0, (*note as f32 - 69.0) / 12.0));
                        outs.value(1)[0] = bool(true);
                        outs.value(2)[0] = float((*velocity as f32) / 127.0);
                    }
                    MidiData::NoteOff { .. } => {
                        outs.value(1)[0] = bool(false);
                    }
                    _ => {}
                }
            }
        }

        NodeOk::no_warnings(())
    }
}

impl Node for MidiToValuesNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        MidiToValuesNode {}
    }

    fn get_io(_context: &NodeGetIoContext, _props: SeaHashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            midi_input("midi", 1),
            value_output("frequency", 1),
            value_output("gate", 1),
            value_output("velocity", 1),
        ])
    }
}
