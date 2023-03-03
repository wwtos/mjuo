use smallvec::SmallVec;
use sound_engine::midi::messages::MidiData;

use crate::connection::{MidiBundle, MidiSocketType, Primitive, ValueSocketType};
use crate::errors::{NodeError, NodeOk};
use crate::node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow};

#[derive(Debug, PartialEq, Clone)]
enum ChangedState {
    NewInfo,
    InfoProcessed,
    NoInfo,
}

#[derive(Debug, Clone)]
pub struct MidiToValuesNode {
    midi_in: MidiBundle,
    frequency: f32,
    gate: bool,
    velocity: f32,
    state: ChangedState,
}

impl Default for MidiToValuesNode {
    fn default() -> Self {
        MidiToValuesNode {
            midi_in: SmallVec::new(),
            frequency: 440.0,
            gate: false,
            velocity: 0.0,
            state: ChangedState::NewInfo,
        }
    }
}

impl Node for MidiToValuesNode {
    fn accept_midi_input(&mut self, _socket_type: MidiSocketType, value: MidiBundle) {
        self.midi_in = value;
        self.state = ChangedState::NewInfo;
    }

    fn process(&mut self, _state: NodeProcessState) -> Result<NodeOk<()>, NodeError> {
        if self.state == ChangedState::NewInfo {
            for data in &self.midi_in {
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

            self.midi_in.clear();
        }

        self.state = match self.state {
            ChangedState::NewInfo => ChangedState::InfoProcessed,
            ChangedState::InfoProcessed => ChangedState::NoInfo,
            ChangedState::NoInfo => ChangedState::NoInfo,
        };

        NodeOk::no_warnings(())
    }

    fn init(&mut self, _state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        InitResult::simple(vec![
            NodeRow::MidiInput(MidiSocketType::Default, SmallVec::new(), false),
            NodeRow::ValueOutput(ValueSocketType::Frequency, Primitive::Float(440.0), false),
            NodeRow::ValueOutput(ValueSocketType::Gate, Primitive::Boolean(false), false),
        ])
    }

    fn get_value_output(&self, socket_type: ValueSocketType) -> Option<Primitive> {
        if self.state == ChangedState::NoInfo {
            return None;
        }

        match socket_type {
            ValueSocketType::Frequency => Some(Primitive::Float(self.frequency)),
            ValueSocketType::Gate => Some(Primitive::Boolean(self.gate)),
            _ => None,
        }
    }
}
