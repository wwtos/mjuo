use std::collections::HashMap;

use rhai::Engine;
use sound_engine::midi::messages::MidiData;

use crate::connection::MidiSocketType;
use crate::node::{InitResult, Node, NodeRow};
use crate::property::Property;
use crate::socket_registry::SocketRegistry;

#[derive(Debug, Default)]
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

    fn init(
        &mut self,
        _properties: &HashMap<String, Property>,
        _registry: &mut SocketRegistry,
        _scripting_engine: &Engine,
    ) -> InitResult {
        InitResult::simple(vec![NodeRow::MidiOutput(MidiSocketType::Default, vec![])])
    }
}
