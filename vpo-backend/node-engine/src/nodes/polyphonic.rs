use sound_engine::constants::SAMPLE_RATE;
use sound_engine::midi::messages::MidiData;

use crate::connection::{MidiSocketType, SocketDirection, SocketType, StreamSocketType};
use crate::errors::{NodeError, NodeOk};
use crate::node::{InitResult, Node, NodeIndex, NodeInitState, NodeProcessState, NodeRow};
use crate::node_graph::NodeGraph;
use crate::property::PropertyType;
use crate::traversal::traverser::Traverser;
use crate::{property::Property, socket_registry::SocketRegistry};

const DIFFERENCE_THRESHOLD: f32 = 0.007;
//                                                             50 ms
const SAME_VALUE_LENGTH_THRESHOLD: i64 = (SAMPLE_RATE / 1000 * 50) as i64;

#[derive(Debug, Clone)]
struct PolyphonicInfo {
    duration_of_same_output: i64,
    last_output_value: f32,
    started_at: i64,
    active: bool,
    note: u8,
}

impl PolyphonicInfo {
    pub fn new(started_at: i64) -> PolyphonicInfo {
        PolyphonicInfo {
            duration_of_same_output: 0,
            last_output_value: 0.0,
            started_at: started_at,
            active: false,
            note: 255,
        }
    }
}

#[derive(Debug, Clone)]
struct Voice {
    graph: NodeGraph,
    info: PolyphonicInfo,
}

#[derive(Debug, Clone)]
pub struct PolyphonicNode {
    voices: Vec<Voice>,
    polyphony: u8,
    traverser: Traverser,
    output: f32,
    is_first_time: bool,
    inner_inputs_node: NodeIndex,
    inner_outputs_node: NodeIndex,
    current_time: i64,
}

impl Default for PolyphonicNode {
    fn default() -> PolyphonicNode {
        PolyphonicNode {
            voices: vec![],
            traverser: Traverser::default(),
            polyphony: 1,
            output: 0_f32,
            is_first_time: true,
            inner_inputs_node: NodeIndex {
                index: 0,
                generation: 0,
            },
            inner_outputs_node: NodeIndex {
                index: 0,
                generation: 0,
            },
            current_time: 0,
        }
    }
}

impl Node for PolyphonicNode {
    fn init(&mut self, state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        if let Some(Property::Integer(polyphony)) = state.props.get("polyphony") {
            self.polyphony = (*polyphony).clamp(1, 255) as u8;
        }

        // TODO: this is pretty hacky
        if self.voices.len() > 0 {
            for i in 0..self.polyphony {
                if i as usize >= self.voices.len() {
                    self.voices.push(Voice {
                        graph: self.voices[0].graph.clone(),
                        info: PolyphonicInfo::new(self.current_time),
                    });
                } else {
                    self.voices[i as usize] = Voice {
                        graph: self.voices[0].graph.clone(),
                        info: PolyphonicInfo::new(self.current_time),
                    };
                }
            }

            if self.voices.len() > self.polyphony as usize {
                self.voices.truncate(self.polyphony as usize);
            }
        }

        InitResult::simple(vec![
            NodeRow::MidiInput(MidiSocketType::Default, vec![]),
            NodeRow::Property("polyphony".to_string(), PropertyType::Integer, Property::Integer(1)),
            NodeRow::InnerGraph,
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0),
        ])
    }

    fn get_inner_graph_socket_list(&self, _registry: &mut SocketRegistry) -> Vec<(SocketType, SocketDirection)> {
        vec![
            (SocketType::Midi(MidiSocketType::Default), SocketDirection::Input),
            (SocketType::Stream(StreamSocketType::Audio), SocketDirection::Output),
        ]
    }

    fn accept_midi_input(&mut self, _socket_type: &MidiSocketType, value: Vec<MidiData>) {
        // go through all the messages and send them to all the appropriate locations
        for message in value {
            let message_to_pass_to_all = match message {
                MidiData::NoteOff { note, .. } => {
                    // look to see if there's a note on for this one, send it the turn off
                    // message if so
                    for voice in self.voices.iter_mut() {
                        if voice.info.active && voice.info.note == note {
                            let subgraph_input_node = voice.graph.get_node_mut(&self.inner_inputs_node).unwrap();
                            subgraph_input_node.accept_midi_input(&MidiSocketType::Default, vec![message]);

                            voice.info.active = true;
                            voice.info.note = note;
                            break;
                        }
                    }

                    None
                }
                MidiData::NoteOn { note, channel, .. } => {
                    println!("note on: {}", note);

                    // search through for a open voice

                    // first, check if there's already one on for this note
                    let already_on = self
                        .voices
                        .iter_mut()
                        .find(|voice| voice.info.active && voice.info.note == note);
                    if let Some(already_on) = already_on {
                        let subgraph_input_node = already_on.graph.get_node_mut(&self.inner_inputs_node).unwrap();

                        // be sure to send a note off message first
                        subgraph_input_node.accept_midi_input(
                            &MidiSocketType::Default,
                            vec![
                                MidiData::NoteOff {
                                    channel: channel,
                                    note,
                                    velocity: 0,
                                },
                                message,
                            ],
                        );

                        already_on.info.active = true;
                        already_on.info.note = note;
                        already_on.info.started_at = self.current_time;
                    } else {
                        // if not, check if one is available
                        let available = self.voices.iter_mut().find(|voice| !voice.info.active);

                        if let Some(available) = available {
                            let subgraph_input_node = available.graph.get_node_mut(&self.inner_inputs_node).unwrap();

                            // TODO: test code here VV
                            subgraph_input_node.accept_midi_input(
                                &MidiSocketType::Default,
                                vec![
                                    MidiData::NoteOff {
                                        channel: channel,
                                        note: available.info.note,
                                        velocity: 0,
                                    },
                                    message,
                                ],
                            );

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

                            let subgraph_input_node = oldest.graph.get_node_mut(&self.inner_inputs_node).unwrap();

                            // be sure to send a note off message first
                            subgraph_input_node.accept_midi_input(
                                &MidiSocketType::Default,
                                vec![
                                    MidiData::NoteOff {
                                        channel: channel,
                                        note: oldest.info.note,
                                        velocity: 0,
                                    },
                                    message,
                                ],
                            );

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
                        let subgraph_input_node = voice.graph.get_node_mut(&self.inner_inputs_node).unwrap();
                        subgraph_input_node
                            .accept_midi_input(&MidiSocketType::Default, vec![message_to_pass_to_all.clone()]);
                    }
                }
            }
        }
    }

    fn get_stream_output(&self, _socket_type: &StreamSocketType) -> f32 {
        self.output
    }

    fn init_graph(&mut self, graph: &mut NodeGraph, input_node: NodeIndex, output_node: NodeIndex) {
        for i in 0..self.polyphony {
            if i as usize >= self.voices.len() {
                self.voices.push(Voice {
                    graph: graph.clone(),
                    info: PolyphonicInfo::new(self.current_time),
                });
            } else {
                self.voices[i as usize] = Voice {
                    graph: graph.clone(),
                    info: PolyphonicInfo::new(self.current_time),
                };
            }
        }

        if self.voices.len() > self.polyphony as usize {
            self.voices.truncate(self.polyphony as usize);
        }

        self.traverser = Traverser::get_traverser(graph);
        self.is_first_time = true;
        self.inner_inputs_node = input_node;
        self.inner_outputs_node = output_node;
    }

    fn process(&mut self, state: NodeProcessState) -> Result<NodeOk<()>, NodeError> {
        self.current_time = state.current_time;

        self.output = 0.0;

        for voice in self.voices.iter_mut() {
            if voice.info.active {
                self.traverser.traverse(
                    &mut voice.graph,
                    self.is_first_time,
                    state.current_time,
                    state.script_engine,
                );

                let subgraph_output_node = voice.graph.get_node_mut(&self.inner_outputs_node).unwrap();
                let output = subgraph_output_node.get_stream_output(&StreamSocketType::Audio);

                self.output += output;

                if (voice.info.last_output_value - output).abs() < DIFFERENCE_THRESHOLD {
                    voice.info.duration_of_same_output += 1;

                    if voice.info.duration_of_same_output > SAME_VALUE_LENGTH_THRESHOLD {
                        // mark voice as inactive
                        voice.info.active = false;
                        voice.info.duration_of_same_output = 0;
                    }
                } else {
                    voice.info.duration_of_same_output = 0;
                    voice.info.last_output_value = output;
                }
            }
        }

        self.is_first_time = false;

        NodeOk::no_warnings(())
    }
}
