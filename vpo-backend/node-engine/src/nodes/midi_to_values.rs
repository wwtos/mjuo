use serde::{Deserialize, Serialize};
use sound_engine::midi::messages::MidiData;

use crate::connection::{MidiSocketType, Primitive, SocketType, ValueSocketType};
use crate::node::Node;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum ChangedState {
    NewInfo,
    InfoProcessed,
    NoInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MidiToValuesNode {
    midi_in: Vec<MidiData>,
    frequency: f32,
    gate: bool,
    velocity: f32,
    state: ChangedState,
}

impl Default for MidiToValuesNode {
    fn default() -> Self {
        MidiToValuesNode {
            midi_in: Vec::new(),
            frequency: 440.0,
            gate: false,
            velocity: 0.0,
            state: ChangedState::NewInfo,
        }
    }
}

impl Node for MidiToValuesNode {
    fn accept_midi_input(&mut self, _socket_type: MidiSocketType, value: Vec<MidiData>) {
        self.midi_in = value;
        self.state = ChangedState::NewInfo;
    }

    fn process(&mut self) {
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

    fn list_input_sockets(&self) -> Vec<SocketType> {
        vec![SocketType::Midi(MidiSocketType::Default)]
    }

    fn list_output_sockets(&self) -> Vec<SocketType> {
        vec![
            SocketType::Value(ValueSocketType::Frequency),
            SocketType::Value(ValueSocketType::Gate),
        ]
    }
}
