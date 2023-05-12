use std::mem;

use smallvec::SmallVec;
use sound_engine::midi::messages::MidiData;

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct MidiMergerNode {
    states: Vec<u128>,
    combined: u128,
    to_send: Option<MidiBundle>,
}

impl MidiMergerNode {
    fn combine(&mut self) {
        let mut sum_state = 0_u128;

        for state in &mut self.states {
            sum_state |= *state;
        }

        self.combined = sum_state;
    }
}

impl NodeRuntime for MidiMergerNode {
    fn init(&mut self, state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        let input_count = state.props.get("input_count").unwrap().as_integer().unwrap();
        self.states.resize(input_count as usize, 0);

        InitResult::nothing()
    }

    fn accept_midi_inputs(&mut self, midi_in: &[Option<MidiBundle>]) {
        let mut new_messages: MidiBundle = SmallVec::new();

        for (i, messages) in midi_in.iter().enumerate() {
            if let Some(messages) = messages {
                for message in messages {
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

                            self.states[i] = !(!self.states[i] | 1_u128 << note);
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
        }

        if !new_messages.is_empty() {
            self.to_send = Some(new_messages);
        }
    }

    fn get_midi_outputs(&mut self, midi_out: &mut [Option<MidiBundle>]) {
        midi_out[0] = mem::replace(&mut self.to_send, None);
    }
}

impl Node for MidiMergerNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        MidiMergerNode {
            states: vec![],
            to_send: None,
            combined: 0,
        }
    }

    fn get_io(props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        let mut node_rows = vec![
            NodeRow::Property("input_count".to_string(), PropertyType::Integer, Property::Integer(2)),
            midi_output(register("midi")),
        ];

        let input_count = props
            .get("input_count")
            .and_then(|x| x.clone().as_integer())
            .unwrap_or(2);

        for i in 0..input_count {
            node_rows.push(NodeRow::Input(
                Socket::Numbered(register("socket-input-numbered"), i + 1, SocketType::Midi, 1),
                SocketValue::None,
            ));
        }

        NodeIo::simple(node_rows)
    }
}
