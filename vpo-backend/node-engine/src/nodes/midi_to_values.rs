use smallvec::SmallVec;
use sound_engine::midi::messages::MidiData;

use crate::nodes::prelude::*;

use super::util::ProcessState;

#[derive(Debug, Clone)]
pub struct MidiToValuesNode {
    frequency: f32,
    gate: bool,
    velocity: f32,
    process_state: ProcessState<()>,
}

impl Default for MidiToValuesNode {
    fn default() -> Self {
        MidiToValuesNode {
            frequency: 440.0,
            gate: false,
            velocity: 0.0,
            process_state: ProcessState::Unprocessed(()),
        }
    }
}

impl NodeRuntime for MidiToValuesNode {
    fn accept_midi_inputs(&mut self, midi_in: &[Option<MidiBundle>]) {
        let midi = midi_in[0].as_ref().unwrap();

        for data in midi {
            match data {
                MidiData::NoteOn {
                    channel: _,
                    note,
                    velocity,
                } => {
                    self.frequency = 440.0 * f32::powf(2.0, (*note as f32 - 69.0) / 12.0);
                    self.velocity = (*velocity as f32) / 127.0;
                    self.gate = true;
                }
                MidiData::NoteOff {
                    channel: _,
                    note: _,
                    velocity: _,
                } => {
                    self.gate = false;
                }
                _ => {}
            }
        }
    }

    fn process(
        &mut self,
        _state: NodeProcessState,
        _streams_in: &[&[f32]],
        _streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        match self.process_state {
            ProcessState::Unprocessed(_) => self.process_state = ProcessState::Processed,
            ProcessState::Processed => self.process_state = ProcessState::None,
            ProcessState::None => {}
        }

        NodeOk::no_warnings(())
    }

    fn get_value_outputs(&self, values_out: &mut [Option<Primitive>]) {
        if matches!(self.process_state, ProcessState::Processed) {
            values_out[0] = Some(Primitive::Float(self.frequency));
            values_out[1] = Some(Primitive::Boolean(self.gate));
        }
    }
}

impl Node for MidiToValuesNode {
    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            midi_input(register("midi"), SmallVec::new()),
            value_output(register("frequency"), Primitive::Float(440.0)),
            value_output(register("gate"), Primitive::Boolean(false)),
        ])
    }
}
