use crate::nodes::prelude::*;

use super::util::is_message_reset;

#[derive(Debug, Clone)]
pub struct MidiToValuesNode {
    freq: f32,
}

impl NodeRuntime for MidiToValuesNode {
    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        midi_store: &mut MidiStore,
        _resources: &[Resource],
    ) {
        if let Some(midi) = &ins.midi(0)[0] {
            let messages = midi_store.borrow_midi(midi).unwrap();

            for data in messages.iter() {
                match &data.data {
                    MidiData::NoteOn {
                        channel: _,
                        note,
                        velocity,
                    } => {
                        self.freq = 440.0 * f32::powf(2.0, (*note as f32 - 69.0) / 12.0);

                        outs.value(0)[0] = float(self.freq);

                        outs.value(1)[0] = bool(true);
                        outs.value(2)[0] = float((*velocity as f32) / 127.0);
                    }
                    MidiData::PitchBend { pitch_bend, .. } => {
                        let bound_bend = (*pitch_bend as i16 - 8192) as f32 / 8192.0;
                        let cents = bound_bend * 200.0;

                        let freq = self.freq * f32::powf(2.0, cents / 1200.0);

                        outs.value(0)[0] = float(freq);
                    }
                    MidiData::NoteOff { .. } => {
                        outs.value(1)[0] = bool(false);
                    }
                    _ => {}
                }

                if is_message_reset(&data.data) {
                    outs.value(0)[0] = float(440.0);

                    outs.value(1)[0] = bool(false);
                    outs.value(2)[0] = float(0.0);
                }
            }
        }
    }
}

impl Node for MidiToValuesNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        MidiToValuesNode { freq: 440.0 }
    }

    fn get_io(_context: NodeGetIoContext, _props: SeaHashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            midi_input("midi", 1),
            value_output("frequency", 1),
            value_output("gate", 1),
            value_output("velocity", 1),
        ])
    }
}
