use sound_engine::midi::messages::MidiData;

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct MidiToValuesNode {}

impl NodeRuntime for MidiToValuesNode {
    fn process(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins,
        outs: Outs,
        _resources: &[Option<(ResourceIndex, &dyn Any)>],
    ) -> NodeResult<()> {
        if let Some(midi) = ins.midis[0] {
            for data in midi {
                match &data.data {
                    MidiData::NoteOn {
                        channel: _,
                        note,
                        velocity,
                    } => {
                        outs.values[0] = float(440.0 * f32::powf(2.0, (*note as f32 - 69.0) / 12.0));
                        outs.values[1] = bool(true);
                        outs.values[2] = float((*velocity as f32) / 127.0);
                    }
                    MidiData::NoteOff { .. } => {
                        outs.values[1] = bool(false);
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

    fn get_io(_props: HashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            midi_input("midi"),
            value_output("frequency"),
            value_output("gate"),
            value_output("velocity"),
        ])
    }
}
