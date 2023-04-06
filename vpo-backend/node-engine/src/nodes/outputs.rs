use crate::nodes::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct OutputsNode {
    values: Vec<Option<Primitive>>,
    midis: Vec<Option<MidiBundle>>,
}

impl NodeRuntime for OutputsNode {
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

    fn init(&mut self, state: NodeInitState, child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        if let Some(Property::SocketList(sockets)) = state.props.get("socket_list") {
            let midi_outputs = sockets
                .iter()
                .filter(|output| output.socket_type() == SocketType::Midi)
                .count();

            let value_outputs = sockets
                .iter()
                .filter(|output| output.socket_type() == SocketType::Value)
                .count();

            self.midis.resize(midi_outputs, None);
            self.values.resize(value_outputs, None);
        }

        InitResult::nothing()
    }
}

impl Node for OutputsNode {
    fn get_io(props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        if let Some(Property::SocketList(sockets)) = props.get("socket_list") {
            NodeIo::simple(
                sockets
                    .iter()
                    .map(|socket_type| NodeRow::from_type_and_direction(socket_type.clone(), SocketDirection::Input))
                    .collect::<Vec<NodeRow>>(),
            )
        } else {
            NodeIo::simple(vec![])
        }
    }
}
