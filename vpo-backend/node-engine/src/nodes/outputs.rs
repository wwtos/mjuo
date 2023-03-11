use crate::{
    connection::{MidiBundle, Primitive, SocketDirection, SocketType},
    errors::{NodeError, NodeOk},
    node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow},
};

#[derive(Debug, Clone, Default)]
pub struct OutputsNode {
    outputs: Vec<SocketType>,
    values: Vec<Option<Primitive>>,
    midis: Vec<Option<MidiBundle>>,
    dirty: bool,
}

impl OutputsNode {
    pub fn set_outputs(&mut self, outputs: Vec<SocketType>) {
        let midi_outputs = outputs
            .iter()
            .filter(|output| matches!(output, SocketType::Midi(_)))
            .count();

        let value_outputs = outputs
            .iter()
            .filter(|output| matches!(output, SocketType::Value(_)))
            .count();

        self.dirty = true;
        self.outputs = outputs;

        self.midis.resize(midi_outputs, None);
        self.values.resize(value_outputs, None);
    }
}

impl Node for OutputsNode {
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

    fn init(&mut self, _state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        let node_rows = self
            .outputs
            .iter()
            .map(|socket_type| NodeRow::from_type_and_direction(socket_type.clone(), SocketDirection::Input, false))
            .collect::<Vec<NodeRow>>();

        NodeOk::no_warnings(InitResult {
            did_rows_change: self.dirty,
            node_rows,
            changed_properties: None,
            child_graph_io: None,
        })
    }
}
