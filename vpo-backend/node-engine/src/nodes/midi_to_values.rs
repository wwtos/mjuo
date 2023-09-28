use sound_engine::midi::messages::MidiData;

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct MidiToValuesNode {}

impl NodeRuntime for MidiToValuesNode {
    fn process<'brand>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'_, 'brand>,
        outs: Outs<'_, 'brand>,
        token: &mut GhostToken<'brand>,
        resources: &[&Resource],
    ) -> NodeResult<()> {
        for data in ins.midis[0][0].borrow(token) {
            match &data.data {
                MidiData::NoteOn {
                    channel: _,
                    note,
                    velocity,
                } => {
                    *outs.values[0][0].borrow_mut(token) = float(440.0 * f32::powf(2.0, (*note as f32 - 69.0) / 12.0));
                    *outs.values[1][0].borrow_mut(token) = bool(true);
                    *outs.values[2][0].borrow_mut(token) = float((*velocity as f32) / 127.0);
                }
                MidiData::NoteOff { .. } => {
                    *outs.values[1][0].borrow_mut(token) = bool(false);
                }
                _ => {}
            }
        }

        NodeOk::no_warnings(())
    }
}

impl Node for MidiToValuesNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        MidiToValuesNode {}
    }

    fn get_io(context: NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            midi_input("midi", 1),
            value_output("frequency", 1),
            value_output("gate", 1),
            value_output("velocity", 1),
        ])
    }
}
