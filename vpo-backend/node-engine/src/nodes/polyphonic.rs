use smallvec::{smallvec, SmallVec};

use sound_engine::midi::messages::{MidiData, MidiMessage};

use crate::{nodes::prelude::*, traversal::buffered_traverser::BufferedTraverser};

use super::variants::NodeVariant;

const DIFFERENCE_THRESHOLD: f32 = 0.007;

#[derive(Debug, Clone)]
struct PolyphonicInfo {
    started_at: i64,
    active: bool,
    note: u8,
}

impl PolyphonicInfo {
    pub fn new(started_at: i64) -> PolyphonicInfo {
        PolyphonicInfo {
            started_at,
            active: false,
            note: 255,
        }
    }
}

#[derive(Debug, Clone)]
struct Voice {
    traverser: BufferedTraverser,
    info: PolyphonicInfo,
    is_first_time: bool,
}

#[derive(Debug, Clone)]
pub struct PolyphonicNode {
    voices: Vec<Voice>,
    polyphony: u8,
    traverser: BufferedTraverser,
    child_io_nodes: Option<(NodeIndex, NodeIndex)>,
    current_time: i64,
}

impl NodeRuntime for PolyphonicNode {
    fn init(&mut self, state: NodeInitState, child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        let mut warnings = vec![];

        if let Some(Property::Integer(polyphony)) = state.props.get("polyphony") {
            self.polyphony = (*polyphony).clamp(1, 255) as u8;
        }

        self.current_time = state.current_time;

        if let Some(graph_and_io) = child_graph {
            self.voices.truncate(self.polyphony as usize);

            while self.polyphony as usize > self.voices.len() {
                let (traverser, errors_and_warnings) = BufferedTraverser::new(
                    graph_and_io.graph,
                    state.graph_manager,
                    state.script_engine,
                    state.resources,
                    state.current_time,
                    state.sound_config.clone(),
                )?;

                if errors_and_warnings.any() {
                    warnings.push(NodeWarning::InternalErrorsAndWarnings { errors_and_warnings });
                }

                self.voices.push(Voice {
                    traverser,
                    info: PolyphonicInfo::new(state.current_time),
                    is_first_time: true,
                });
            }

            self.child_io_nodes = Some((graph_and_io.input_index, graph_and_io.output_index));
        }

        Ok(NodeOk::new(
            InitResult {
                changed_properties: None,
            },
            warnings,
        ))
    }

    fn accept_midi_inputs(&mut self, midi_in: &[Option<MidiBundle>]) {
        let value = midi_in[0].clone().unwrap();

        let (child_input_node, _) = self.child_io_nodes.unwrap();

        // have we created any voices?
        if !self.voices.is_empty() {
            // go through all the messages and send them to all the appropriate locations
            for message in value {
                let message_to_pass_to_all = match message.data {
                    MidiData::NoteOff { note, .. } => {
                        // look to see if there's a note on for this one, send it a turn off
                        // message if so
                        for voice in self.voices.iter_mut() {
                            if voice.info.active && voice.info.note == note {
                                let subgraph_input_node = voice.traverser.get_node_mut(child_input_node).unwrap();
                                subgraph_input_node.accept_midi_inputs(&[Some(smallvec![message])]);

                                voice.info.active = true;
                                voice.info.note = note;
                                break;
                            }
                        }

                        None
                    }
                    MidiData::NoteOn { note, channel, .. } => {
                        // search through for a open voice

                        // first, check if there's already one on for this note
                        let already_on = self
                            .voices
                            .iter_mut()
                            .find(|voice| voice.info.active && voice.info.note == note);
                        if let Some(already_on) = already_on {
                            let subgraph_input_node = already_on.traverser.get_node_mut(child_input_node).unwrap();

                            // be sure to send a note off message first
                            subgraph_input_node.accept_midi_inputs(&[Some(smallvec![
                                MidiMessage {
                                    data: MidiData::NoteOff {
                                        channel,
                                        note,
                                        velocity: 0,
                                    },
                                    timestamp: message.timestamp - 1
                                },
                                message,
                            ])]);

                            already_on.info.active = true;
                            already_on.info.note = note;
                            already_on.info.started_at = self.current_time;
                        } else {
                            // if not, check if one is available
                            let available = self.voices.iter_mut().find(|voice| !voice.info.active);

                            if let Some(available) = available {
                                let subgraph_input_node = available.traverser.get_node_mut(child_input_node).unwrap();

                                // TODO: test code here VV
                                subgraph_input_node.accept_midi_inputs(&[Some(smallvec![
                                    MidiMessage {
                                        data: MidiData::NoteOff {
                                            channel,
                                            note: available.info.note,
                                            velocity: 0,
                                        },
                                        timestamp: message.timestamp - 1
                                    },
                                    message,
                                ])]);

                                available.info.active = true;
                                available.info.note = note;
                                available.info.started_at = self.current_time;
                            } else {
                                // just pick the oldest played note
                                let oldest = self
                                    .voices
                                    .iter_mut()
                                    .min_by(|x, y| x.info.started_at.cmp(&y.info.started_at))
                                    .unwrap();

                                let subgraph_input_node = oldest.traverser.get_node_mut(child_input_node).unwrap();

                                // be sure to send a note off message first
                                subgraph_input_node.accept_midi_inputs(&[Some(smallvec![
                                    MidiMessage {
                                        data: MidiData::NoteOff {
                                            channel,
                                            note: oldest.info.note,
                                            velocity: 0,
                                        },
                                        timestamp: message.timestamp - 1
                                    },
                                    message,
                                ])]);

                                oldest.info.active = true;
                                oldest.info.note = note;
                                oldest.info.started_at = self.current_time;
                            }
                        }

                        None
                    }
                    _ => Some(message),
                };

                // it wasn't note on or note off, so we better make sure all the voices get it
                if let Some(message_to_pass_to_all) = message_to_pass_to_all {
                    for voice in self.voices.iter_mut() {
                        if voice.info.active {
                            let subgraph_input_node = voice.traverser.get_node_mut(child_input_node).unwrap();
                            subgraph_input_node.accept_midi_inputs(&[Some(smallvec![message_to_pass_to_all.clone()])]);
                        }
                    }
                }
            }
        }
    }

    fn process(
        &mut self,
        state: NodeProcessState,
        _streams_in: &[&[f32]],
        streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        let (_, child_output_node) = self.child_io_nodes.unwrap();

        // loop through voices
        for voice in self.voices.iter_mut() {
            if voice.info.active {
                // if it's active, process it
                self.traverser
                    .traverse(state.current_time, state.script_engine, state.resources, vec![], None);

                let subgraph_output_node = voice.traverser.get_node_mut(child_output_node).unwrap();

                let child_output = match subgraph_output_node {
                    NodeVariant::OutputsNode(output) => output.get_streams(),
                    _ => {
                        return Err(NodeError::IncorrectNodeType {
                            expected: "NodeOutputs".into(),
                            actual: format!("{:?}", subgraph_output_node),
                        })
                    }
                };

                for (output, child_output) in streams_out[0].iter_mut().zip(&child_output[0]) {
                    *output += child_output;
                }

                // audio is all less than difference threshold?
                if child_output[0].iter().all(|frame| frame.abs() < DIFFERENCE_THRESHOLD) {
                    // mark voice as inactive
                    voice.info.active = false;
                }

                voice.is_first_time = false;
            }
        }

        NodeOk::no_warnings(())
    }
}

impl Node for PolyphonicNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        PolyphonicNode {
            voices: vec![],
            traverser: BufferedTraverser::default(),
            polyphony: 1,
            child_io_nodes: None,
            current_time: 0,
        }
    }

    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo {
            node_rows: vec![
                midi_input(register("default"), SmallVec::new()),
                NodeRow::Property("polyphony".to_string(), PropertyType::Integer, Property::Integer(1)),
                NodeRow::InnerGraph,
                stream_output(register("audio")),
            ],
            child_graph_io: Some(vec![
                (
                    Socket::Simple(register("midi"), SocketType::Midi, 1),
                    SocketDirection::Input,
                ),
                (
                    Socket::Simple(register("audio"), SocketType::Stream, 1),
                    SocketDirection::Output,
                ),
            ]),
        }
    }
}
