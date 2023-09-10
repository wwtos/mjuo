use sound_engine::midi::messages::{MidiData, MidiMessage};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct MidiTransposeNode {
    transpose_by: i16,
}

impl NodeRuntime for MidiTransposeNode {
    fn process(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins,
        outs: Outs,
        resources: &[&dyn Any],
    ) -> NodeResult<()> {
        if let Some(transpose) = ins.values[0][0].as_int() {
            self.transpose_by = transpose.clamp(-127, 127) as i16;
        }

        outs.midis[0][0] = ins.midis[0][0]
            .iter()
            .filter_map(|message| match message.data {
                MidiData::NoteOn {
                    channel,
                    note,
                    velocity,
                } => {
                    let new_note = (note as i16) + self.transpose_by;

                    if new_note >= 0 && new_note <= 127 {
                        Some(MidiMessage {
                            data: MidiData::NoteOn {
                                channel,
                                note: new_note as u8,
                                velocity,
                            },
                            timestamp: message.timestamp,
                        })
                    } else {
                        None
                    }
                }
                MidiData::NoteOff {
                    channel,
                    note,
                    velocity,
                } => {
                    let new_note = (note as i16) + self.transpose_by;

                    if new_note >= 0 && new_note <= 127 {
                        Some(MidiMessage {
                            data: MidiData::NoteOff {
                                channel,
                                note: new_note as u8,
                                velocity,
                            },
                            timestamp: message.timestamp,
                        })
                    } else {
                        None
                    }
                }
                _ => Some(message.clone()),
            })
            .collect::<MidiBundle>();

        ProcessResult::nothing()
    }
}

impl Node for MidiTransposeNode {
    fn get_io(context: NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            midi_input("midi", 1),
            value_input("transpose", Primitive::Int(0), 1),
            midi_output("midi", 1),
        ])
    }

    fn new(_sound_config: &SoundConfig) -> Self {
        MidiTransposeNode { transpose_by: 0 }
    }
}
