use std::collections::HashMap;

use rhai::Engine;
use sound_engine::constants::SAMPLE_RATE;
use sound_engine::midi::messages::MidiData;

use crate::connection::{MidiSocketType, SocketDirection, SocketType, StreamSocketType};
use crate::errors::ErrorsAndWarnings;
use crate::node::{InitResult, Node, NodeIndex, NodeRow};
use crate::node_graph::NodeGraph;
use crate::property::PropertyType;
use crate::traversal::traverser::Traverser;
use crate::{property::Property, socket_registry::SocketRegistry};

const DIFFERENCE_THRESHOLD: f32 = 0.007;
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

struct Foo {
    graph: NodeGraph,
    info: PolyphonicInfo
}

#[derive(Debug, Clone)]
pub struct PolyphonicNode {
    local_graphs: Vec<Foo>,
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
            local_graphs: vec![],
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
    fn init(
        &mut self,
        properties: &HashMap<String, Property>,
        _registry: &mut SocketRegistry,
        _scripting_engine: &Engine,
    ) -> InitResult {
        if let Some(Property::Integer(polyphony)) = properties.get("polyphony") {
            self.polyphony = (*polyphony).clamp(1, 255) as u8;
        }

        // TODO: this is pretty hacky
        if self.local_graphs.len() > 0 {
            for i in 0..self.polyphony {
                if i as usize >= self.local_graphs.len() {
                    self.local_graphs
                        .push((self.local_graphs[0].graph.clone(), PolyphonicInfo::new(self.current_time)));
                } else {
                    self.local_graphs[i as usize] =
                        (self.local_graphs[0].graph.clone(), PolyphonicInfo::new(self.current_time));
                }
            }

            if self.local_graphs.len() > self.polyphony as usize {
                self.local_graphs.truncate(self.polyphony as usize);
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
                    for voice in self.local_graphs.iter_mut() {
                        if voice.1.active && voice.1.note == note {
                            let subgraph_input_node = voice.0.get_node_mut(&self.inner_inputs_node).unwrap();
                            subgraph_input_node.accept_midi_input(&MidiSocketType::Default, vec![message]);

                            voice.1.active = true;
                            voice.1.note = note;
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
                        .local_graphs
                        .iter_mut()
                        .find(|voice| voice.1.active && voice.1.note == note);
                    if let Some(already_on) = already_on {
                        let subgraph_input_node = already_on.0.get_node_mut(&self.inner_inputs_node).unwrap();

                        // be sure to send a note off message first
                        subgraph_input_node.accept_midi_input(
                            &MidiSocketType::Default,
                            vec![MidiData::NoteOff {
                                channel: channel,
                                note,
                                velocity: 0,
                            }, message],
                        );

                        already_on.1.active = true;
                        already_on.1.note = note;
                        already_on.1.started_at = self.current_time;
                    } else {
                        // if not, check if one is available
                        let available = self.local_graphs.iter_mut().find(|voice| !voice.1.active);

                        if let Some(available) = available {
                            let subgraph_input_node = available.0.get_node_mut(&self.inner_inputs_node).unwrap();

                            // TODO: test code here VV
                            subgraph_input_node.accept_midi_input(
                                &MidiSocketType::Default,
                                vec![MidiData::NoteOff {
                                    channel: channel,
                                    note: available.1.note,
                                    velocity: 0,
                                }, message],
                            );

                            available.1.active = true;
                            available.1.note = note;
                            available.1.started_at = self.current_time;
                        } else {
                            // just pick the oldest played note
                            let oldest = self
                                .local_graphs
                                .iter_mut()
                                .min_by(|x, y| x.1.started_at.cmp(&y.1.started_at))
                                .unwrap();

                            let subgraph_input_node = oldest.0.get_node_mut(&self.inner_inputs_node).unwrap();

                            // be sure to send a note off message first
                            subgraph_input_node.accept_midi_input(
                                &MidiSocketType::Default,
                                vec![MidiData::NoteOff {
                                    channel: channel,
                                    note: oldest.1.note,
                                    velocity: 0,
                                }, message],
                            );

                            oldest.1.active = true;
                            oldest.1.note = note;
                            oldest.1.started_at = self.current_time;
                        }
                    }

                    None
                }
                _ => Some(message),
            };

            // it wasn't note on or note off, so we better make sure all the voices get it
            if let Some(message_to_pass_to_all) = message_to_pass_to_all {
                for voice in self.local_graphs.iter_mut() {
                    if voice.1.active {
                        let subgraph_input_node = voice.0.get_node_mut(&self.inner_inputs_node).unwrap();
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
            if i as usize >= self.local_graphs.len() {
                self.local_graphs
                    .push((graph.clone(), PolyphonicInfo::new(self.current_time)));
            } else {
                self.local_graphs[i as usize] = (graph.clone(), PolyphonicInfo::new(self.current_time));
            }
        }

        if self.local_graphs.len() > self.polyphony as usize {
            self.local_graphs.truncate(self.polyphony as usize);
        }

        self.traverser = Traverser::get_traverser(graph);
        self.is_first_time = true;
        self.inner_inputs_node = input_node;
        self.inner_outputs_node = output_node;
    }

    fn process(
        &mut self,
        current_time: i64,
        scripting_engine: &Engine,
        _inner_graph: Option<(&mut NodeGraph, &Traverser)>,
    ) -> Result<(), ErrorsAndWarnings> {
        let mut errors_and_warnings = ErrorsAndWarnings::default();

        self.current_time = current_time;

        self.output = 0.0;

        for voice in self.local_graphs.iter_mut() {
            if voice.1.active {
                errors_and_warnings = errors_and_warnings.merge(self.traverser.traverse(
                    &mut voice.0,
                    self.is_first_time,
                    current_time,
                    scripting_engine,
                ))?;

                let subgraph_output_node = voice.0.get_node_mut(&self.inner_outputs_node).unwrap();
                let output = subgraph_output_node.get_stream_output(&StreamSocketType::Audio);

                self.output += output;

                if (voice.1.last_output_value - output).abs() < DIFFERENCE_THRESHOLD {
                    voice.1.duration_of_same_output += 1;

                    if voice.1.duration_of_same_output > SAME_VALUE_LENGTH_THRESHOLD {
                        // mark voice as inactive
                        voice.1.active = false;
                    }
                } else {
                    voice.1.duration_of_same_output = 0;
                    voice.1.last_output_value = output;
                }
            }
        }

        self.is_first_time = false;

        Ok(())
    }
}
