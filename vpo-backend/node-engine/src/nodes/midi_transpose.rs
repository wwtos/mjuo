use sound_engine::midi::messages::{MidiData, MidiMessage};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct MidiTransposeNode {
    transpose_by: i16,
}

impl NodeRuntime for MidiTransposeNode {
    fn process<'a, 'arena: 'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a, 'arena>,
        mut outs: Outs<'a, 'arena>,
        arena: &'arena BuddyArena,
        resources: &[&Resource],
    ) -> NodeResult<()> {
        if let Some(transpose) = ins.value(0)[0].as_int() {
            self.transpose_by = transpose.clamp(-127, 127) as i16;
        }

        if let Some(midi) = ins.midi(0)[0] {
            let output: Vec<MidiMessage> = midi
                .value
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
                .collect();

            outs.midi(0)[0] = arena.alloc_slice_fill_iter(output.into_iter()).ok();
        }

        ProcessResult::nothing()
    }
}

impl Node for MidiTransposeNode {
    fn get_io(context: &NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
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
