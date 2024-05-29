use std::borrow::Cow;

use common::osc_midi::{NOTE_OFF_C, NOTE_ON_C};

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct NoteMergerNode {
    states: Vec<u128>,
    combined: u128,
    scratch: Vec<u8>,
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
        let input_count = params.props.get_int("input_count")?;
        self.states.resize(input_count.max(2) as usize, 0);

        InitResult::nothing()
    }

    fn process<'a>(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        osc_store: &mut OscStore,
        _resources: &[Resource],
    ) {
        self.scratch.clear();

        for (i, possible_msgs) in ins.oscs().enumerate() {
            let Some(messages) = possible_msgs[0]
                .get_messages(osc_store)
                .and_then(|bytes| OscView::new(bytes))
            else {
                continue;
            };

            messages.all_messages(|_, _, message| {
                let addr = message.address();

                if addr == NOTE_ON_C {
                    let Some((_, note, _)) = read_osc!(message.arg_iter(), as_int, as_int, as_int) else {
                        return;
                    };

                    let before = self.combined;

                    self.states[i] |= 1_u128 << note;
                    self.combine();

                    // the state changed, so we should pass this message through
                    if self.combined != before {
                        write_message(&mut self.scratch, message);
                    }
                } else if addr == NOTE_OFF_C {
                    let Some((_, note, _)) = read_osc!(message.arg_iter(), as_int, as_int, as_int) else {
                        return;
                    };

                    let before = self.combined;

                    self.states[i] &= !(1_u128 << note);
                    self.combine();

                    // the state changed, so we should pass this message through
                    if self.combined != before {
                        write_message(&mut self.scratch, message);
                    }
                } else {
                    write_message(&mut self.scratch, message);
                }
            });
        }

        outs.osc(0)[0] = write_bundle_and_message_scratch(osc_store, &self.scratch);
    }
}

impl Node for NoteMergerNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        NoteMergerNode {
            states: vec![],
            scratch: default_osc(),
            combined: 0,
        }
    }

    fn get_io(_context: NodeGetIoContext, props: SeaHashMap<String, Property>) -> NodeIo {
        let mut node_rows = vec![
            NodeRow::Property("input_count".to_string(), PropertyType::Integer, Property::Integer(2)),
            osc_output("midi", 1),
        ];

        // TODO: upgrade to add inputs based on how many are connected
        let input_count = props
            .get("input_count")
            .and_then(|x| x.clone().as_integer())
            .unwrap_or(2)
            .max(2);

        for i in 0..input_count {
            node_rows.push(NodeRow::Input(
                Socket::WithData(Cow::Borrowed("input_numbered"), (i + 1).to_string(), SocketType::Osc, 1),
                SocketValue::None,
            ));
        }

        NodeIo::simple(node_rows)
    }
}
