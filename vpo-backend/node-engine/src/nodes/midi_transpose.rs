use common::osc_midi::{NOTE_OFF_C, NOTE_ON_C};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct MidiTransposeNode {
    transpose_by: i16,
    scratch: Vec<u8>,
    currently_on: u128,
}

impl NodeRuntime for MidiTransposeNode {
    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        osc_store: &mut OscStore,
        _resources: &[Resource],
    ) {
        self.scratch.clear();
        let messages = ins.osc(0)[0]
            .get_messages(osc_store)
            .and_then(|bytes| OscView::new(bytes));

        if let Some(transpose) = ins.value(0)[0].as_int() {
            let last_on = self.currently_on;
            let last_transpose_by = self.transpose_by;
            self.transpose_by = transpose.clamp(-127, 127) as i16;

            let shift = self.transpose_by - last_transpose_by;
            let new_on = signed_left_shift(last_on, shift);

            let difference = last_on ^ new_on;

            for i in 0..128 {
                let did_note_change = ((difference >> i) & 0x01) != 0x00;

                if !did_note_change {
                    continue;
                }

                let is_note_on = ((new_on >> i) & 0x01) != 0x00;

                if is_note_on {
                    write_note_on(&mut self.scratch, 0, i, 127);
                } else {
                    write_note_off(&mut self.scratch, 0, i, 0);
                }
            }
        }

        if let Some(messages) = messages {
            messages.all_messages(|_, _, message| {
                if message.address() == NOTE_ON_C {
                    let Some((channel, note, velocity)) = read_osc!(message.arg_iter(), as_int, as_int, as_int) else {
                        return;
                    };

                    let new_note = (note as i16) + self.transpose_by;

                    if new_note >= 0 && new_note <= 127 {
                        self.currently_on |= 1_u128 << new_note;
                        write_note_on(&mut self.scratch, channel as u8, new_note as u8, velocity as u8);
                    }
                } else if message.address() == NOTE_OFF_C {
                    let Some((channel, note, velocity)) = read_osc!(message.arg_iter(), as_int, as_int, as_int) else {
                        return;
                    };

                    let new_note = (note as i16) + self.transpose_by;

                    if new_note >= 0 && new_note <= 127 {
                        self.currently_on &= !(1_u128 << new_note);
                        write_note_off(&mut self.scratch, channel as u8, new_note as u8, velocity as u8);
                    }
                } else {
                    write_message(&mut self.scratch, message);
                }
            });
        }

        outs.osc(0)[0] = write_bundle_and_message_scratch(osc_store, &self.scratch);
    }
}

fn signed_left_shift(num: u128, shift: i16) -> u128 {
    if shift >= 0 {
        num << shift
    } else {
        num >> (-shift)
    }
}

impl Node for MidiTransposeNode {
    fn get_io(_context: NodeGetIoContext, _props: SeaHashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            midi_input("midi", 1),
            value_input("transpose", int(0), 1),
            midi_output("midi", 1),
        ])
    }

    fn new(_sound_config: &SoundConfig) -> Self {
        MidiTransposeNode {
            transpose_by: 0,
            scratch: default_osc(),
            currently_on: 0x00,
        }
    }
}
