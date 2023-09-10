use crate::nodes::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct InputsNode {
    values: Vec<Option<Primitive>>,
    midis: Vec<Option<MidiBundle>>,
    sent: bool,
}

impl InputsNode {
    pub fn set_values(&mut self, values: Vec<Option<Primitive>>) {
        self.values = values;
        self.sent = false;
    }

    pub fn set_midis(&mut self, midis: Vec<Option<MidiBundle>>) {
        self.midis = midis;
        self.sent = false;
    }
}

impl NodeRuntime for InputsNode {
    fn process(
        &mut self,
        _context: NodeProcessContext,
        _ins: Ins,
        outs: Outs,
        _resources: &[Option<(ResourceIndex, &dyn Any)>],
    ) -> NodeResult<()> {
        if !self.sent {
            outs.midis.clone_from_slice(&self.midis[..]);
            outs.values.clone_from_slice(&self.values[..]);

            self.sent = true;
        }

        NodeOk::no_warnings(())
    }
}

impl Node for InputsNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        InputsNode {
            values: vec![],
            midis: vec![],
            sent: false,
        }
    }

    fn get_io(props: HashMap<String, Property>) -> NodeIo {
        if let Some(Property::SocketList(sockets)) = props.get("socket_list") {
            NodeIo::simple(
                sockets
                    .iter()
                    .map(|socket| NodeRow::from_type_and_direction(socket.clone(), SocketDirection::Output))
                    .collect::<Vec<NodeRow>>(),
            )
        } else {
            NodeIo::simple(vec![])
        }
    }
}
