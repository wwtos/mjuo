use std::mem;

use smallvec::SmallVec;
use sound_engine::midi::messages::{MidiData, MidiMessage};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct MidiTransposeNode {
    transpose_by: i16,
    midi_out: Option<MidiBundle>,
}

impl NodeRuntime for MidiTransposeNode {
    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        if let Some(value_in) = values_in[0].as_ref().and_then(|value_in| value_in.as_int()) {
            self.transpose_by = value_in.clamp(-127, 127) as i16;
        }
    }

    fn accept_midi_inputs(&mut self, midi_in: &[Option<MidiBundle>]) {
        let midi_in = midi_in[0].as_ref().unwrap();

        self.midi_out = Some(
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

    fn get_midi_outputs(&mut self, midi_out: &mut [Option<MidiBundle>]) {
        midi_out[0] = mem::replace(&mut self.midi_out, None);
    }
}

impl Node for MidiTransposeNode {
    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            midi_input(register("midi"), SmallVec::new()),
            value_input(register("transpose"), Primitive::Int(0)),
            midi_output(register("midi")),
        ])
    }

    fn new(_sound_config: &SoundConfig) -> Self {
        MidiTransposeNode {
            transpose_by: 0,
            midi_out: None,
        }
    }
}
