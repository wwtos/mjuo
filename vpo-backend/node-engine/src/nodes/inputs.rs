use crate::{
    connection::{MidiBundle, Primitive, SocketDirection, SocketType},
    errors::{NodeError, NodeOk},
    node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow},
};
#[derive(Debug, Clone, Default)]
pub struct InputsNode {
    inputs: Vec<SocketType>,
    values: Vec<Option<Primitive>>,
    midis: Vec<Option<MidiBundle>>,
    dirty: bool,
}

impl InputsNode {
    pub fn set_inputs(&mut self, inputs: Vec<SocketType>) {
        let midi_inputs = inputs
            .iter()
            .filter(|input| matches!(input, SocketType::Midi(_)))
            .count();

        let value_inputs = inputs
            .iter()
            .filter(|input| matches!(input, SocketType::Value(_)))
            .count();

        self.dirty = true;
        self.inputs = inputs;

        self.midis.resize(midi_inputs, None);
        self.values.resize(value_inputs, None);
    }
}

impl Node for InputsNode {
    fn init(&mut self, _state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        let node_rows = self
            .inputs
            .iter()
            .map(|socket_type| NodeRow::from_type_and_direction(socket_type.clone(), SocketDirection::Output, false))
            .collect::<Vec<NodeRow>>();

        let was_dirty = self.dirty;
        self.dirty = false;

        NodeOk::no_warnings(InitResult {
            did_rows_change: was_dirty,
            node_rows,
            changed_properties: None,
        })
    }

    fn process(
        &mut self,
        _state: NodeProcessState,
        streams_in: &[f32],
        streams_out: &mut [f32],
    ) -> Result<NodeOk<()>, NodeError> {
        streams_out.clone_from_slice(streams_in);

        NodeOk::no_warnings(())
    }

    fn accept_midi_inputs(&mut self, midi_in: &[Option<MidiBundle>]) {
        self.midis.clone_from_slice(midi_in);
    }

    fn get_midi_outputs(&self, midi_out: &mut [Option<MidiBundle>]) {
        midi_out.clone_from_slice(&self.midis[..]);
    }

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        self.values.clone_from_slice(values_in);
    }

    fn get_value_outputs(&self, values_out: &mut [Option<Primitive>]) {
        values_out.clone_from_slice(&self.values[..]);
    }
}
