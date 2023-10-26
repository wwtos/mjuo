use crate::nodes::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct InputsNode {
    values: Vec<Primitive>,
    midis: Vec<MidiBundle>,
    sent: bool,
}

impl InputsNode {
    pub fn set_values(&mut self, values: Vec<Primitive>) {
        self.values = values;
        self.sent = false;
    }

    pub fn set_midis(&mut self, midis: Vec<MidiBundle>) {
        self.midis = midis;
        self.sent = false;
    }
}

impl NodeRuntime for InputsNode {
    fn process<'a, 'arena: 'a, 'brand>(
        &mut self,
        _context: NodeProcessContext,
        _ins: Ins<'a, 'arena, 'brand>,
        outs: Outs<'a, 'arena, 'brand>,
        token: &mut GhostToken<'brand>,
        arena: &'arena BuddyArena,
        _resources: &[&Resource],
    ) -> NodeResult<()> {
        if !self.sent {
            for (message_out, message_in) in outs.midis[0].iter().zip(self.midis.drain(..)) {
                *message_out.borrow_mut(token) = arena.alloc_slice_fill_iter(message_in.into_iter()).ok();
            }

            for (values_out, value_to_output) in outs.values.iter().zip(self.values.iter()) {
                values_out[0].set(*value_to_output);
            }

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

    fn get_io(context: &NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
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
