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
pub struct OutputsNode {
    outputs: Vec<SocketType>,
    values_in: Vec<ProcessState<SocketValue>>,
    values_out: Vec<Option<SocketValue>>,
    dirty: bool,
}

impl OutputsNode {
    pub fn set_outputs(&mut self, outputs: Vec<SocketType>) {
        self.dirty = true;
        self.outputs = outputs;

        for i in 0..self.values_in.len().min(self.outputs.len()) {
            self.values_in[i] = ProcessState::Unprocessed(match self.outputs[i] {
                SocketType::Stream(_) => SocketValue::Stream(0.0),
                SocketType::Midi(_) => SocketValue::Midi(SmallVec::new()),
                SocketType::Value(_) => SocketValue::Value(Primitive::Boolean(false)),
                SocketType::NodeRef(_) => SocketValue::NodeRef,
                SocketType::MethodCall(_) => todo!(),
            });
        }

        for i in self.values_in.len()..self.outputs.len() {
            self.values_in.push(ProcessState::Unprocessed(match self.outputs[i] {
                SocketType::Stream(_) => SocketValue::Stream(0.0),
                SocketType::Midi(_) => SocketValue::Midi(SmallVec::new()),
                SocketType::Value(_) => SocketValue::Value(Primitive::Boolean(false)),
                SocketType::NodeRef(_) => SocketValue::NodeRef,
                SocketType::MethodCall(_) => todo!(),
            }));
        }

        if self.values_in.len() > self.outputs.len() {
            self.values_in.truncate(self.outputs.len());
        }

        self.transfer_inputs_to_outputs();
    }

    fn transfer_inputs_to_outputs(&mut self) {
        self.values_out = self
            .values_in
            .iter_mut()
            .map(|value_in| {
                let (new_value, new_state) = match value_in {
                    ProcessState::Unprocessed(new_value) => (Some(new_value.clone()), ProcessState::Processed),
                    ProcessState::Processed => (None, ProcessState::None),
                    ProcessState::None => (None, ProcessState::None),
                };

                *value_in = new_state;

                new_value
            })
            .collect();
    }
}

impl Node for OutputsNode {
    fn process(&mut self, _state: NodeProcessState) -> Result<NodeOk<()>, NodeError> {
        self.transfer_inputs_to_outputs();

        NodeOk::no_warnings(())
    }

    fn accept_stream_input(&mut self, socket_type: &StreamSocketType, value: f32) {
        let index = self
            .outputs
            .iter()
            .position(|x| x == &SocketType::Stream(socket_type.clone()))
            .unwrap();

        self.values_in[index] = ProcessState::Unprocessed(SocketValue::Stream(value));
    }

    fn accept_midi_input(&mut self, socket_type: &MidiSocketType, value: MidiBundle) {
        let index = self
            .outputs
            .iter()
            .position(|x| x == &SocketType::Midi(socket_type.clone()))
            .unwrap();

        self.values_in[index] = ProcessState::Unprocessed(SocketValue::Midi(value));
    }

    fn accept_value_input(&mut self, socket_type: &ValueSocketType, value: Primitive) {
        let index = self
            .outputs
            .iter()
            .position(|x| x == &SocketType::Value(socket_type.clone()))
            .unwrap();

        self.values_in[index] = ProcessState::Unprocessed(SocketValue::Value(value));
    }

    fn get_stream_output(&self, socket_type: &StreamSocketType) -> f32 {
        let index = self
            .outputs
            .iter()
            .position(|x| x == &SocketType::Stream(socket_type.clone()))
            .unwrap();

        self.values_out[index].clone().unwrap().as_stream().unwrap()
    }

    fn get_midi_output(&self, socket_type: &MidiSocketType) -> Option<MidiBundle> {
        let index = self
            .outputs
            .iter()
            .position(|x| x == &SocketType::Midi(socket_type.clone()))
            .unwrap();

        if let Some(midi_out) = &self.values_out[index] {
            Some(midi_out.clone().as_midi().unwrap())
        } else {
            None
        }
    }

    fn get_value_output(&self, socket_type: &ValueSocketType) -> Option<Primitive> {
        let index = self
            .outputs
            .iter()
            .position(|x| x == &SocketType::Value(socket_type.clone()))
            .unwrap();

        self.values_out[index]
            .as_ref()
            .map(|value_out| value_out.clone().as_value().unwrap())
    }

    fn init(&mut self, _state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        let node_rows = self
            .outputs
            .iter()
            .map(|socket_type| NodeRow::from_type_and_direction(socket_type.clone(), SocketDirection::Input))
            .collect::<Vec<NodeRow>>();

        NodeOk::no_warnings(InitResult {
            did_rows_change: self.dirty,
            node_rows,
            changed_properties: None,
        })
    }
}
