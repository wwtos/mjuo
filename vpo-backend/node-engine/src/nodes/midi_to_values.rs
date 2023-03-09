use smallvec::SmallVec;
use sound_engine::midi::messages::MidiData;

use crate::connection::{MidiBundle, MidiSocketType, Primitive, ValueSocketType};
use crate::errors::{NodeError, NodeOk};
use crate::node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow};

use super::util::ProcessState;

#[derive(Debug, Clone)]
pub struct MidiToValuesNode {
    midi_in: ProcessState<MidiBundle>,
    frequency: f32,
    gate: bool,
    velocity: f32,
}

impl Default for MidiToValuesNode {
    fn default() -> Self {
        MidiToValuesNode {
            midi_in: ProcessState::None,
            frequency: 440.0,
            gate: false,
            velocity: 0.0,
        }
    }
}

impl Node for MidiToValuesNode {
    fn accept_midi_inputs(&mut self, midi_in: &[Option<MidiBundle>]) {
        self.midi_in = ProcessState::Unprocessed(midi_in[0].unwrap());
    }

    fn process(
        &mut self,
        state: NodeProcessState,
        streams_in: &[f32],
        streams_out: &mut [f32],
    ) -> Result<NodeOk<()>, NodeError> {
        match self.midi_in {
            ProcessState::Unprocessed(midi_in) => {
                for data in midi_in {
                    match data {
                        MidiData::NoteOn {
                            channel: _,
                            note,
                            velocity,
                        } => {
                            self.frequency = 440.0 * f32::powf(2.0, (note as f32 - 69.0) / 12.0);
                            self.velocity = (velocity as f32) / 127.0;
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
            ProcessState::Processed => self.midi_in = ProcessState::None,
            ProcessState::None => {}
        }

        NodeOk::no_warnings(())
    }

    fn init(&mut self, _state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        InitResult::simple(vec![
            NodeRow::MidiInput(MidiSocketType::Default, SmallVec::new(), false),
            NodeRow::ValueOutput(ValueSocketType::Frequency, Primitive::Float(440.0), false),
            NodeRow::ValueOutput(ValueSocketType::Gate, Primitive::Boolean(false), false),
        ])
    }

    fn get_value_outputs(&self, values_out: &mut [Option<Primitive>]) {
        if matches!(self.midi_in, ProcessState::Processed) {
            values_out[0] = Some(Primitive::Float(self.frequency));
            values_out[1] = Some(Primitive::Boolean(self.gate));
        }
    }
}
