use smallvec::{smallvec, SmallVec};
use sound_engine::midi::messages::MidiData;
use web_sys::console;

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct SequencerNode {
    last_emitted_at: i64,
    to_emit: Option<MidiBundle>,
    chord_note: u8,
    active: bool,
    resetting: bool,
}

impl Default for SequencerNode {
    fn default() -> Self {
        SequencerNode {
            last_emitted_at: 0,
            to_emit: None,
            chord_note: 50,
            active: false,
            resetting: false,
        }
    }
}

impl NodeRuntime for SequencerNode {
    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        let active = values_in[0].clone().unwrap().as_boolean().unwrap();

        if active == false {
            self.to_emit = Some(smallvec![
                MidiData::NoteOff {
                    channel: 0,
                    note: self.chord_note,
                    velocity: 127
                },
                MidiData::NoteOff {
                    channel: 0,
                    note: self.chord_note + 4,
                    velocity: 127
                },
                MidiData::NoteOff {
                    channel: 0,
                    note: self.chord_note + 7,
                    velocity: 127
                }
            ]);

            self.resetting = true;
        }

        self.active = active;
        self.chord_note = 50;
    }

    fn process(
        &mut self,
        state: NodeProcessState,
        _streams_in: &[&[f32]],
        _streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        if self.resetting {
            self.resetting = false;

            return NodeOk::no_warnings(());
        }

        if state.current_time - self.last_emitted_at > 48_000 && self.active {
            self.last_emitted_at = state.current_time;
            self.chord_note += 1;

            self.to_emit = Some(smallvec![
                MidiData::NoteOff {
                    channel: 0,
                    note: self.chord_note - 1,
                    velocity: 127
                },
                MidiData::NoteOff {
                    channel: 0,
                    note: self.chord_note + 4 - 1,
                    velocity: 127
                },
                MidiData::NoteOff {
                    channel: 0,
                    note: self.chord_note + 7 - 1,
                    velocity: 127
                },
                MidiData::NoteOn {
                    channel: 0,
                    note: self.chord_note,
                    velocity: 127
                },
                MidiData::NoteOn {
                    channel: 0,
                    note: self.chord_note + 4,
                    velocity: 127
                },
                MidiData::NoteOn {
                    channel: 0,
                    note: self.chord_note + 7,
                    velocity: 127
                }
            ]);

            console::log_1(&format!("emitting {:?}", self.to_emit).into());
        } else {
            self.to_emit = None;
        }

        NodeOk::no_warnings(())
    }

    fn get_midi_outputs(&self, midi_out: &mut [Option<MidiBundle>]) {
        midi_out[0] = self.to_emit.clone();
    }
}

impl Node for SequencerNode {
    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            value_input(register("active"), Primitive::Boolean(false)),
            midi_output(register("midi"), SmallVec::new()),
        ])
    }
}
