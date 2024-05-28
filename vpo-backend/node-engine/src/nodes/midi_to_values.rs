use common::osc_midi::is_message_reset;

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct MidiToValuesNode {
    base_freq: f32,
    pitch_bend: f32,
}

impl NodeRuntime for MidiToValuesNode {
    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        osc_store: &mut OscStore,
        _resources: &[Resource],
    ) {
        let Some(messages) = &ins.osc(0)[0]
            .get_messages(osc_store)
            .and_then(|bytes| OscView::new(bytes))
        else {
            return;
        };

        messages.all_messages(|_, _, message| {
            match message.address().to_str() {
                Ok(NOTE_ON) => {
                    let Some((_, note, velocity)) = read_osc!(message.arg_iter(), as_int, as_int, as_int) else {
                        return;
                    };

                    self.base_freq = 440.0 * f32::powf(2.0, (note as f32 - 69.0) / 12.0);

                    outs.value(0)[0] = float(self.base_freq * self.pitch_bend);
                    outs.value(1)[0] = bool(true);
                    outs.value(2)[0] = float((velocity as f32) / 127.0);
                }
                Ok(NOTE_OFF) => {
                    outs.value(1)[0] = bool(false);
                }
                Ok(PITCH_BEND) => {
                    let Some((_, bend)) = read_osc!(message.arg_iter(), as_int, as_int) else {
                        return;
                    };

                    let bound_bend = (bend - 8192) as f32 / 8192.0;
                    let cents = bound_bend * 200.0;

                    self.pitch_bend = f32::powf(2.0, cents / 1200.0);

                    outs.value(0)[0] = float(self.base_freq * self.pitch_bend);
                }
                _ => {}
            }

            if is_message_reset(message) {
                outs.value(0)[0] = float(440.0);

                outs.value(1)[0] = bool(false);
                outs.value(2)[0] = float(0.0);
            }
        });
    }
}

impl Node for MidiToValuesNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        MidiToValuesNode {
            base_freq: 440.0,
            pitch_bend: 1.0,
        }
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
