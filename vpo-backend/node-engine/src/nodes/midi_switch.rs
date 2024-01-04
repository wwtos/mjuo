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
}

impl NodeRuntime for MidiSwitchNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        if let Some(Property::String(mode)) = params.props.get("mode") {
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

    fn process<'a>(
        &mut self,
        context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        midi_store: &mut MidiStoreInterface,
        _resources: &[Resource],
    ) -> NodeResult<()> {
        let mut midi_out: MidiChannel = MidiChannel::new();

        if let Some(midi) = &ins.midi(0)[0] {
            let messages = midi_store.borrow_midi(midi).unwrap();

            for message in messages.iter() {
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
        }

        if let Some(engaged) = ins.value(0)[0].as_boolean() {
            // if it's the same value as last time, ignore it
            if engaged != self.engaged {
                self.engaged = engaged;
                let mut midi_out: MidiChannel = MidiChannel::new();

                if engaged {
                    match self.mode {
                        SwitchMode::Normal => {
                            // turn on all the notes that are pressed
                            for i in 0..128 {
                                if self.state & (1 << i) != 0 {
                                    midi_out.push(MidiMessage {
                                        timestamp: context.current_time,
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
                                timestamp: context.current_time,
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
            }
        }

        if !midi_out.is_empty() {
            outs.midi(0)[0] = midi_store.register_midis(midi_out.into_iter());
        }

        ProcessResult::nothing()
    }
}

impl Node for MidiSwitchNode {
    fn get_io(_context: &NodeGetIoContext, _props: SeaHashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            midi_input("midi", 1),
            value_input("engage", Primitive::Boolean(false), 1),
            multiple_choice("mode", &["normal", "sostenuto", "sustain"], "normal"),
            midi_output("midi", 1),
        ])
    }

    fn new(_sound_config: &SoundConfig) -> Self {
        MidiSwitchNode {
            mode: SwitchMode::Normal,
            state: 0,
            ignoring: 0,
            engaged: false,
        }
    }
}
