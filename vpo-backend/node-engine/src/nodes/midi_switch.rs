use std::mem;

use smallvec::smallvec;
use sound_engine::midi::messages::{MidiData, MidiMessage};

use super::prelude::*;

#[derive(Debug, Clone)]
enum SwitchMode {
    Normal,
    Sostenuto,
    Sustain,
}

#[derive(Debug, Clone)]
pub struct MidiSwitchNode {
    mode: SwitchMode,
    state: u128,
    ignoring: u128,
    engaged: bool,
    midi_out: Option<MidiBundle>,
}

impl NodeRuntime for MidiSwitchNode {
    fn init(&mut self, state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        if let Some(Property::String(mode)) = state.props.get("mode") {
            self.ignoring = 0;

            match mode.as_str() {
                "normal" => {
                    self.mode = SwitchMode::Normal;
                }
                "sostenuto" => {
                    self.mode = SwitchMode::Sostenuto;
                }
                "sustain" => {
                    self.mode = SwitchMode::Sustain;
                }
                _ => {
                    self.mode = SwitchMode::Normal;
                }
            };
        }

        InitResult::nothing()
    }

    fn accept_midi_inputs(&mut self, midi_in: &[Option<MidiBundle>]) {
        let midi_in = midi_in[0].as_ref().expect("a midi input");
        let mut midi_out: MidiBundle = smallvec![];

        for message in midi_in {
            match message.data {
                MidiData::NoteOn { note, .. } => {
                    match self.mode {
                        SwitchMode::Normal => {
                            if self.engaged {
                                midi_out.push(message.clone());
                            }
                        }
                        SwitchMode::Sostenuto => {
                            // is the note not being ignored?
                            if (1_128 << note) & self.ignoring == 0 {
                                midi_out.push(message.clone());
                            }
                        }
                        SwitchMode::Sustain => {
                            midi_out.push(message.clone());
                        }
                    }

                    self.state |= 1 << note;
                }
                MidiData::NoteOff { note, .. } => {
                    match self.mode {
                        SwitchMode::Normal => {
                            if self.engaged {
                                midi_out.push(message.clone());
                            }
                        }
                        SwitchMode::Sostenuto => {
                            let being_ignored = (1 << note) & self.ignoring != 0;

                            if !being_ignored {
                                midi_out.push(message.clone());
                            }
                        }
                        SwitchMode::Sustain => {
                            // if it's engaged, don't pass note off messages
                            if !self.engaged {
                                midi_out.push(message.clone());
                            }
                        }
                    }

                    self.state &= !(1 << note);
                }
                _ => {
                    if self.engaged {
                        midi_out.push(message.clone());
                    }
                }
            }
        }

        if !midi_out.is_empty() {
            if let Some(current_midi_out) = &mut self.midi_out {
                current_midi_out.extend(midi_out.into_iter());
            } else {
                self.midi_out = Some(midi_out);
            }
        }
    }

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        if let Some(engaged) = values_in[0].as_ref().and_then(|x| x.as_boolean()) {
            // if it's the same value as last time, ignore it
            if engaged == self.engaged {
                return;
            }

            self.engaged = engaged;
            let mut midi_out: MidiBundle = smallvec![];

            if engaged {
                match self.mode {
                    SwitchMode::Normal => {
                        // turn on all the notes that are pressed
                        for i in 0..128 {
                            if self.state & (1 << i) != 0 {
                                midi_out.push(MidiMessage {
                                    timestamp: 0,
                                    data: MidiData::NoteOn {
                                        channel: 0,
                                        note: i,
                                        velocity: 0,
                                    },
                                })
                            }
                        }
                    }
                    SwitchMode::Sostenuto => {
                        self.ignoring = self.state;
                    }
                    SwitchMode::Sustain => {}
                }
            } else {
                let to_turn_off = self.state | self.ignoring;

                for i in 0..128 {
                    if to_turn_off & (1 << i) != 0 {
                        midi_out.push(MidiMessage {
                            timestamp: 0,
                            data: MidiData::NoteOff {
                                channel: 0,
                                note: i,
                                velocity: 0,
                            },
                        })
                    }
                }

                self.state = 0;
                self.ignoring = 0;
            }

            if !midi_out.is_empty() {
                if let Some(current_midi_out) = &mut self.midi_out {
                    current_midi_out.extend(midi_out.into_iter());
                } else {
                    self.midi_out = Some(midi_out);
                }
            }
        }
    }

    fn get_midi_outputs(&mut self, midi_out: &mut [Option<MidiBundle>]) {
        midi_out[0] = mem::replace(&mut self.midi_out, None);
    }
}

impl Node for MidiSwitchNode {
    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            midi_input(register("midi")),
            value_input(register("engage"), Primitive::Boolean(false)),
            multiple_choice("mode", &["normal", "sostenuto", "sustain"], "normal"),
            midi_output(register("midi")),
        ])
    }

    fn new(_sound_config: &SoundConfig) -> Self {
        MidiSwitchNode {
            mode: SwitchMode::Normal,
            state: 0,
            ignoring: 0,
            engaged: false,
            midi_out: None,
        }
    }
}
