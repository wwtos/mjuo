use common::osc_midi::{NOTE_OFF_C, NOTE_ON_C};

use super::prelude::*;

#[derive(Debug, Clone)]
enum SwitchMode {
    Normal,
    Sostenuto,
    Sustain,
}

#[derive(Debug, Clone)]
pub struct MidiSwitchNode {
    scratch: Vec<u8>,
    mode: SwitchMode,
    state: u128,
    ignoring: u128,
    engaged: bool,
}

impl NodeRuntime for MidiSwitchNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let mode = params.props.get_string("mode")?;

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

        InitResult::nothing()
    }

    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        osc_store: &mut OscStore,
        _resources: &[Resource],
    ) {
        self.scratch.clear();

        let messages_in = ins.osc(0)[0]
            .get_messages(osc_store)
            .and_then(|bytes| OscView::new(bytes));

        if let Some(messages) = messages_in {
            messages.all_messages(|_, _, message| {
                let addr = message.address();
                let args = message.arg_iter();

                if addr == NOTE_ON_C {
                    let Some((_channel, note, _velocity)) = read_osc!(args, as_int, as_int, as_int) else {
                        return;
                    };

                    match self.mode {
                        SwitchMode::Normal => {
                            if self.engaged {
                                write_message(&mut self.scratch, message);
                            }
                        }
                        SwitchMode::Sostenuto => {
                            // is the note not being ignored?
                            if (1_128 << note) & self.ignoring == 0 {
                                write_message(&mut self.scratch, message);
                            }
                        }
                        SwitchMode::Sustain => {
                            write_message(&mut self.scratch, message);
                        }
                    }

                    self.state |= 1 << note;
                } else if addr == NOTE_OFF_C {
                    let Some((_channel, note, _velocity)) = read_osc!(args, as_int, as_int, as_int) else {
                        return;
                    };

                    match self.mode {
                        SwitchMode::Normal => {
                            if self.engaged {
                                write_message(&mut self.scratch, message);
                            }
                        }
                        SwitchMode::Sostenuto => {
                            let being_ignored = (1 << note) & self.ignoring != 0;

                            if !being_ignored {
                                write_message(&mut self.scratch, message);
                            }
                        }
                        SwitchMode::Sustain => {
                            // if it's engaged, don't pass note off messages
                            if !self.engaged {
                                write_message(&mut self.scratch, message);
                            }
                        }
                    }

                    self.state &= !(1 << note);
                } else {
                    if self.engaged {
                        write_message(&mut self.scratch, message);
                    }
                }
            });
        }

        if let Some(engaged) = ins.value(0)[0].as_boolean() {
            // if it's the same value as last time, ignore it
            if engaged != self.engaged {
                self.engaged = engaged;

                if engaged {
                    match self.mode {
                        SwitchMode::Normal => {
                            // send note on for all the notes that are already pressed
                            for i in 0..128 {
                                if self.state & (1 << i) != 0 {
                                    write_note_on(&mut self.scratch, 0, i, 127);
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
                            write_note_off(&mut self.scratch, 0, i, 0);
                        }
                    }

                    self.ignoring = 0;
                }
            }
        }

        outs.osc(0)[0] = write_bundle_and_message_scratch(osc_store, &self.scratch);
    }
}

impl Node for MidiSwitchNode {
    fn get_io(_context: NodeGetIoContext, _props: SeaHashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            midi_input("midi", 1),
            value_input("engage", Primitive::Boolean(false), 1),
            multiple_choice("mode", &["normal", "sostenuto", "sustain"], "normal"),
            midi_output("midi", 1),
        ])
    }

    fn new(_sound_config: &SoundConfig) -> Self {
        MidiSwitchNode {
            scratch: default_osc(),
            mode: SwitchMode::Normal,
            state: 0,
            ignoring: 0,
            engaged: false,
        }
    }
}
