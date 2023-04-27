use crate::nodes::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct InputsNode {
    values: Vec<Option<Primitive>>,
    midis: Vec<Option<MidiBundle>>,
}

impl NodeRuntime for InputsNode {
    fn process(
        &mut self,
        _state: NodeProcessState,
        _streams_in: &[&[f32]],
        _streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
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

impl Node for InputsNode {
    fn new(sound_config: &SoundConfig) -> Self {
        InputsNode {
            values: vec![],
            midis: vec![],
        }
    }

    fn get_io(props: HashMap<String, Property>, _register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        if let Some(Property::SocketList(sockets)) = props.get("socket_list") {
            NodeIo::simple(
                sockets
                    .iter()
                    .map(|socket_type| NodeRow::from_type_and_direction(socket_type.clone(), SocketDirection::Output))
                    .collect::<Vec<NodeRow>>(),
            )
        } else {
            NodeIo::simple(vec![])
        }
    }
}
