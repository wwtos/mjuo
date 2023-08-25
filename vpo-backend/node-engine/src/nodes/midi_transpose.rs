use sound_engine::midi::messages::{MidiData, MidiMessage};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct MidiTransposeNode {
    transpose_by: i16,
}

impl NodeRuntime for MidiTransposeNode {
    fn process(
        &mut self,
        globals: NodeProcessGlobals,
        ins: Ins,
        outs: Outs,
        resources: &[(ResourceIndex, &dyn Any)],
    ) -> NodeResult<()> {
        if let Some(transpose) = ins.values[0].as_ref().and_then(|value_in| value_in.as_int()) {
            self.transpose_by = transpose.clamp(-127, 127) as i16;
        }

        if let Some(midi_in) = ins.midis[0] {
            outs.midis[0] = Some(
                midi_in
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
                    .collect(),
            );
        }

        ProcessResult::nothing()
    }
}

impl Node for MidiTransposeNode {
    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            midi_input(register("midi")),
            value_input(register("transpose"), Primitive::Int(0)),
            midi_output(register("midi")),
        ])
    }

    fn new(_sound_config: &SoundConfig) -> Self {
        MidiTransposeNode { transpose_by: 0 }
    }
}
