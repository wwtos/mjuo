use std::borrow::Cow;

use sound_engine::midi::messages::MidiData;

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct NoteMergerNode {
    states: Vec<u128>,
    combined: u128,
}

impl NoteMergerNode {
    fn combine(&mut self) {
        let mut sum_state = 0_u128;

        for state in &mut self.states {
            sum_state |= *state;
        }

        self.combined = sum_state;
    }
}

impl NodeRuntime for NoteMergerNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let input_count = params.props.get("input_count").unwrap().as_integer().unwrap();
        self.states.resize(input_count.max(2) as usize, 0);

        InitResult::nothing()
    }

    fn process<'brand>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'_, 'brand>,
        outs: Outs<'_, 'brand>,
        token: &mut GhostToken<'brand>,
        resources: &[&Resource],
    ) -> NodeResult<()> {
        let mut new_messages: MidiBundle = MidiBundle::new();

        for (i, messages) in ins.midis.iter().enumerate() {
            for message in messages[0].borrow(token) {
                match message.data {
                    MidiData::NoteOn { note, .. } => {
                        let before = self.combined;

                        self.states[i] |= 1_u128 << note;
                        self.combine();

                        // the state changed, so we should pass this message through
                        if self.combined != before {
                            new_messages.push(message.clone());
                        }
                    }
                    MidiData::NoteOff { note, .. } => {
                        let before = self.combined;

                        self.states[i] &= !(1_u128 << note);
                        self.combine();

                        // the state changed, so we should pass this message through
                        if self.combined != before {
                            new_messages.push(message.clone());
                        }
                    }
                    _ => {
                        new_messages.push(message.clone());
                    }
                }
            }
        }

        *outs.midis[0][0].borrow_mut(token) = new_messages;

        ProcessResult::nothing()
    }
}

impl Node for NoteMergerNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        NoteMergerNode {
            states: vec![],
            combined: 0,
        }
    }

    fn get_io(context: NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
        let mut node_rows = vec![
            NodeRow::Property("input_count".to_string(), PropertyType::Integer, Property::Integer(2)),
            midi_output("midi", 1),
        ];

        let input_count = props
            .get("input_count")
            .and_then(|x| x.clone().as_integer())
            .unwrap_or(2)
            .max(2);

        for i in 0..input_count {
            node_rows.push(NodeRow::Input(
                Socket::WithData(
                    Cow::Borrowed("input_numbered"),
                    (i + 1).to_string(),
                    SocketType::Midi,
                    1,
                ),
                SocketValue::None,
            ));
        }

        NodeIo::simple(node_rows)
    }
}
