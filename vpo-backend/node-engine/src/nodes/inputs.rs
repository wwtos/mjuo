use smallvec::SmallVec;

use crate::{
    connection::{
        MidiBundle, MidiSocketType, Primitive, SocketDirection, SocketType, SocketValue, StreamSocketType,
        ValueSocketType,
    },
    errors::{NodeError, NodeOk},
    node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow},
};

use super::util::ProcessState;

#[derive(Debug, Clone, Default)]
pub struct InputsNode {
    inputs: Vec<SocketType>,
    values: Vec<SocketValue>,
    value_changed: Vec<ProcessState<()>>,
    dirty: bool,
}

impl InputsNode {
    pub fn set_inputs(&mut self, inputs: Vec<SocketType>) {
        self.dirty = true;
        self.inputs = inputs;

        for i in 0..self.values.len().min(self.inputs.len()) {
            self.values[i] = match self.inputs[i] {
                SocketType::Stream(_) => SocketValue::Stream(0.0),
                SocketType::Midi(_) => SocketValue::Midi(SmallVec::new()),
                SocketType::Value(_) => SocketValue::Value(Primitive::Boolean(false)),
                SocketType::NodeRef(_) => SocketValue::NodeRef,
            };

            self.value_changed[i] = ProcessState::Unprocessed(());
        }

        for i in self.values.len()..self.inputs.len() {
            self.values.push(match self.inputs[i] {
                SocketType::Stream(_) => SocketValue::Stream(0.0),
                SocketType::Midi(_) => SocketValue::Midi(SmallVec::new()),
                SocketType::Value(_) => SocketValue::Value(Primitive::Boolean(false)),
                SocketType::NodeRef(_) => SocketValue::NodeRef,
            });

            self.value_changed.push(ProcessState::Unprocessed(()));
        }

        if self.values.len() > self.inputs.len() {
            self.values.truncate(self.inputs.len());
            self.value_changed.truncate(self.inputs.len());
        }
    }
}

impl Node for InputsNode {
    fn process(&mut self, _state: NodeProcessState) -> Result<NodeOk<()>, NodeError> {
        for value_changed in self.value_changed.iter_mut() {
            match value_changed {
                ProcessState::Unprocessed(_) => {
                    *value_changed = ProcessState::Processed;
                }
                ProcessState::Processed => {
                    *value_changed = ProcessState::None;
                }
                ProcessState::None => {}
            }
        }

        NodeOk::no_warnings(())
    }

    fn accept_stream_input(&mut self, socket_type: StreamSocketType, value: f32) {
        let index = self
            .inputs
            .iter()
            .position(|x| x == &SocketType::Stream(socket_type.clone()))
            .unwrap();

        self.values[index] = SocketValue::Stream(value);
    }

    fn accept_midi_input(&mut self, socket_type: MidiSocketType, value: MidiBundle) {
        let index = self
            .inputs
            .iter()
            .position(|x| x == &SocketType::Midi(socket_type.clone()))
            .unwrap();

        if !value.is_empty() {
            self.values[index] = SocketValue::Midi(value);
            self.value_changed[index] = ProcessState::Unprocessed(());
        } else {
            self.value_changed[index] = ProcessState::None;
        }
    }

    fn accept_value_input(&mut self, socket_type: ValueSocketType, value: Primitive) {
        let index = self
            .inputs
            .iter()
            .position(|x| x == &SocketType::Value(socket_type.clone()))
            .unwrap();

        self.values[index] = SocketValue::Value(value);
        self.value_changed[index] = ProcessState::Unprocessed(());
    }

    fn get_stream_output(&self, socket_type: StreamSocketType) -> f32 {
        let index = self
            .inputs
            .iter()
            .position(|x| x == &SocketType::Stream(socket_type.clone()))
            .unwrap();

        self.values[index].clone().as_stream().unwrap()
    }

    fn get_midi_output(&self, socket_type: MidiSocketType) -> Option<MidiBundle> {
        let index = self
            .inputs
            .iter()
            .position(|x| x == &SocketType::Midi(socket_type.clone()))
            .unwrap();

        if matches!(self.value_changed[index], ProcessState::Processed) {
            Some(self.values[index].clone().as_midi().unwrap())
        } else {
            None
        }
    }

    fn get_value_output(&self, socket_type: ValueSocketType) -> Option<Primitive> {
        let index = self
            .inputs
            .iter()
            .position(|x| x == &SocketType::Value(socket_type.clone()))
            .unwrap();

        if matches!(self.value_changed[index], ProcessState::Processed) {
            Some(self.values[index].clone().as_value().unwrap())
        } else {
            None
        }
    }

    fn init(&mut self, _state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        let node_rows = self
            .inputs
            .iter()
            .map(|socket_type| NodeRow::from_type_and_direction(socket_type.clone(), SocketDirection::Output, false))
            .collect::<Vec<NodeRow>>();

        NodeOk::no_warnings(InitResult {
            did_rows_change: self.dirty,
            node_rows,
            changed_properties: None,
        })
    }
}
