use std::collections::HashMap;

use rhai::Engine;
use sound_engine::midi::messages::MidiData;

use crate::{
    connection::{
        MidiSocketType, Primitive, SocketDirection, SocketType, SocketValue, StreamSocketType, ValueSocketType,
    },
    node::{InitResult, Node, NodeRow},
    property::Property,
    socket_registry::SocketRegistry,
};

#[derive(Debug, Clone, Default)]
pub struct OutputsNode {
    outputs: Vec<SocketType>,
    values: Vec<SocketValue>,
    value_changed: Vec<bool>,
    dirty: bool,
}

impl OutputsNode {
    pub fn set_outputs(&mut self, outputs: Vec<SocketType>) {
        self.dirty = true;
        self.outputs = outputs;

        for i in 0..self.values.len().min(self.outputs.len()) {
            self.values[i] = match self.outputs[i] {
                SocketType::Stream(_) => SocketValue::Stream(0.0),
                SocketType::Midi(_) => SocketValue::Midi(vec![]),
                SocketType::Value(_) => SocketValue::Value(Primitive::Boolean(false)),
                SocketType::NodeRef(_) => SocketValue::NodeRef,
                SocketType::MethodCall(_) => todo!(),
            };
        }

        for i in self.values.len()..self.outputs.len() {
            self.values.push(match self.outputs[i] {
                SocketType::Stream(_) => SocketValue::Stream(0.0),
                SocketType::Midi(_) => SocketValue::Midi(vec![]),
                SocketType::Value(_) => SocketValue::Value(Primitive::Boolean(false)),
                SocketType::NodeRef(_) => SocketValue::NodeRef,
                SocketType::MethodCall(_) => todo!(),
            });
        }

        if self.values.len() > self.outputs.len() {
            self.values.truncate(self.outputs.len());
        }
    }
}

impl Node for OutputsNode {
    fn accept_stream_input(&mut self, socket_type: &StreamSocketType, value: f32) {
        let index = self
            .outputs
            .iter()
            .position(|x| x == &SocketType::Stream(socket_type.clone()))
            .unwrap();

        self.values[index] = SocketValue::Stream(value);
    }

    fn accept_midi_input(&mut self, socket_type: &MidiSocketType, value: Vec<MidiData>) {
        let index = self
            .outputs
            .iter()
            .position(|x| x == &SocketType::Midi(socket_type.clone()))
            .unwrap();

        self.values[index] = SocketValue::Midi(value);
        self.value_changed[index] = true;
    }

    fn accept_value_input(&mut self, socket_type: &ValueSocketType, value: Primitive) {
        let index = self
            .outputs
            .iter()
            .position(|x| x == &SocketType::Value(socket_type.clone()))
            .unwrap();

        self.values[index] = SocketValue::Value(value);
        self.value_changed[index] = true;
    }

    fn get_stream_output(&self, socket_type: &StreamSocketType) -> f32 {
        let index = self
            .outputs
            .iter()
            .position(|x| x == &SocketType::Stream(socket_type.clone()))
            .unwrap();

        self.values[index].clone().as_stream().unwrap()
    }

    fn get_midi_output(&self, socket_type: &MidiSocketType) -> Vec<MidiData> {
        let index = self
            .outputs
            .iter()
            .position(|x| x == &SocketType::Midi(socket_type.clone()))
            .unwrap();

        if self.value_changed[index] {
            self.values[index].clone().as_midi().unwrap()
        } else {
            vec![]
        }
    }

    fn get_value_output(&self, socket_type: &ValueSocketType) -> Option<Primitive> {
        let index = self
            .outputs
            .iter()
            .position(|x| x == &SocketType::Value(socket_type.clone()))
            .unwrap();

        if self.value_changed[index] {
            Some(self.values[index].clone().as_value().unwrap())
        } else {
            None
        }
    }

    fn init(
        &mut self,
        _properties: &HashMap<String, Property>,
        _registry: &mut SocketRegistry,
        _scripting_engine: &Engine,
    ) -> InitResult {
        let node_rows = self
            .outputs
            .iter()
            .map(|socket_type| NodeRow::from_type_and_direction(socket_type.clone(), SocketDirection::Input))
            .collect::<Vec<NodeRow>>();

        InitResult {
            did_rows_change: self.dirty,
            node_rows: node_rows,
            changed_properties: None,
            errors_and_warnings: None,
        }
    }
}
