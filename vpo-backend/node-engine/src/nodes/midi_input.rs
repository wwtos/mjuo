use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sound_engine::midi::messages::MidiData;

use crate::connection::MidiSocketType;
use crate::node::{InitResult, Node, NodeRow};
use crate::property::Property;
use crate::socket_registry::SocketRegistry;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MidiInNode {
    midi_in: Vec<MidiData>,
}

impl Node for MidiInNode {
    fn accept_midi_input(&mut self, _socket_type: &MidiSocketType, value: Vec<MidiData>) {
        self.midi_in = value;
    }

    fn get_midi_output(&self, _socket_type: &MidiSocketType) -> Vec<MidiData> {
        self.midi_in.clone()
    }

    fn init(&mut self, _properties: &HashMap<String, Property>, _registry: &mut SocketRegistry) -> InitResult {
        InitResult {
            did_rows_change: false,
            node_rows: vec![NodeRow::MidiOutput(MidiSocketType::Default, vec![])],
            changed_properties: None,
        }
    }
}
