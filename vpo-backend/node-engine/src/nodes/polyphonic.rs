use smallvec::SmallVec;
use sound_engine::SoundConfig;

use crate::connection::{MidiBundle, MidiSocketType, SocketDirection, SocketType, StreamSocketType};
use crate::errors::{NodeError, NodeOk, NodeResult};
use crate::node::{InitResult, Node, NodeGraphAndIo, NodeIndex, NodeInitState, NodeProcessState, NodeRow};
use crate::node_graph::NodeGraph;
use crate::property::Property;
use crate::property::PropertyType;
use crate::traversal::traverser::Traverser;

const DIFFERENCE_THRESHOLD: f32 = 0.007;
const SAME_VALUE_LENGTH_THRESHOLD: u32 = 50; // ms

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
            started_at,
            active: false,
            note: 255,
        }
    }
}

#[derive(Debug, Clone)]
struct Voice {
    graph: NodeGraph,
    info: PolyphonicInfo,
    is_first_time: bool,
}

#[derive(Debug, Clone)]
pub struct PolyphonicNode {
    same_value_length_threshold: i64,
    voices: Vec<Voice>,
    polyphony: u8,
    traverser: Traverser,
    child_io_nodes: Option<(NodeIndex, NodeIndex)>,
    current_time: i64,
}

impl PolyphonicNode {
    pub fn new(sound_config: &SoundConfig) -> PolyphonicNode {
        PolyphonicNode {
            same_value_length_threshold: (sound_config.sample_rate / 1000 * SAME_VALUE_LENGTH_THRESHOLD) as i64,
            voices: vec![],
            traverser: Traverser::default(),
            polyphony: 1,
            child_io_nodes: None,
            current_time: 0,
        }
    }
}

impl Node for PolyphonicNode {
    fn init(&mut self, state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        // if let Some(Property::Integer(polyphony)) = state.props.get("polyphony") {
        //     self.polyphony = (*polyphony).clamp(1, 255) as u8;
        // }

        // // TODO: this is pretty hacky
        // if !self.voices.is_empty() {
        //     for i in 0..self.polyphony {
        //         if i as usize >= self.voices.len() {
        //             self.voices.push(Voice {
        //                 graph: self.voices[0].graph.clone(),
        //                 info: PolyphonicInfo::new(self.current_time),
        //                 is_first_time: true,
        //             });
        //         } else {
        //             self.voices[i as usize] = Voice {
        //                 graph: self.voices[0].graph.clone(),
        //                 info: PolyphonicInfo::new(self.current_time),
        //                 is_first_time: true,
        //             };
        //         }
        //     }

        //     if self.voices.len() > self.polyphony as usize {
        //         self.voices.truncate(self.polyphony as usize);
        //     }
        // }

        NodeOk::no_warnings(InitResult {
            did_rows_change: false,
            node_rows: vec![
                NodeRow::MidiInput(MidiSocketType::Default, SmallVec::new(), false),
                NodeRow::Property("polyphony".to_string(), PropertyType::Integer, Property::Integer(1)),
                NodeRow::InnerGraph,
                NodeRow::StreamOutput(StreamSocketType::Audio, 0.0, false),
            ],
            changed_properties: None,
            child_graph_io: Some(vec![
                (SocketType::Midi(MidiSocketType::Default), SocketDirection::Input),
                (SocketType::Stream(StreamSocketType::Audio), SocketDirection::Output),
            ]),
        })
    }

    fn accept_midi_inputs(&mut self, midi_in: &[Option<MidiBundle>]) {
        // let value = midi_in[0].unwrap();

        // let (child_input_node, _) = self.child_io_nodes.unwrap();

        // if !self.voices.is_empty() {
        //     // go through all the messages and send them to all the appropriate locations
        //     for message in value {
        //         let message_to_pass_to_all = match message {
        //             MidiData::NoteOff { note, .. } => {
        //                 // look to see if there's a note on for this one, send it the turn off
        //                 // message if so
        //                 for voice in self.voices.iter_mut() {
        //                     if voice.info.active && voice.info.note == note {
        //                         let subgraph_input_node = voice.graph.get_node_mut(child_input_node).unwrap();
        //                         subgraph_input_node.accept_midi_input(MidiSocketType::Default, smallvec![message]);

        //                         voice.info.active = true;
        //                         voice.info.note = note;
        //                         break;
        //                     }
        //                 }

        //                 None
        //             }
        //             MidiData::NoteOn { note, channel, .. } => {
        //                 // search through for a open voice

        //                 // first, check if there's already one on for this note
        //                 let already_on = self
        //                     .voices
        //                     .iter_mut()
        //                     .find(|voice| voice.info.active && voice.info.note == note);
        //                 if let Some(already_on) = already_on {
        //                     let subgraph_input_node = already_on.graph.get_node_mut(child_input_node).unwrap();

        //                     // be sure to send a note off message first
        //                     subgraph_input_node.accept_midi_input(
        //                         MidiSocketType::Default,
        //                         smallvec![
        //                             MidiData::NoteOff {
        //                                 channel,
        //                                 note,
        //                                 velocity: 0,
        //                             },
        //                             message,
        //                         ],
        //                     );

        //                     already_on.info.active = true;
        //                     already_on.info.note = note;
        //                     already_on.info.started_at = self.current_time;
        //                 } else {
        //                     // if not, check if one is available
        //                     let available = self.voices.iter_mut().find(|voice| !voice.info.active);

        //                     if let Some(available) = available {
        //                         let subgraph_input_node = available.graph.get_node_mut(child_input_node).unwrap();

        //                         // TODO: test code here VV
        //                         subgraph_input_node.accept_midi_input(
        //                             MidiSocketType::Default,
        //                             smallvec![
        //                                 MidiData::NoteOff {
        //                                     channel,
        //                                     note: available.info.note,
        //                                     velocity: 0,
        //                                 },
        //                                 message,
        //                             ],
        //                         );

        //                         available.info.active = true;
        //                         available.info.note = note;
        //                         available.info.started_at = self.current_time;
        //                     } else {
        //                         // just pick the oldest played note
        //                         let oldest = self
        //                             .voices
        //                             .iter_mut()
        //                             .min_by(|x, y| x.info.started_at.cmp(&y.info.started_at))
        //                             .unwrap();

        //                         let subgraph_input_node = oldest.graph.get_node_mut(child_input_node).unwrap();

        //                         // be sure to send a note off message first
        //                         subgraph_input_node.accept_midi_input(
        //                             MidiSocketType::Default,
        //                             smallvec![
        //                                 MidiData::NoteOff {
        //                                     channel,
        //                                     note: oldest.info.note,
        //                                     velocity: 0,
        //                                 },
        //                                 message,
        //                             ],
        //                         );

        //                         oldest.info.active = true;
        //                         oldest.info.note = note;
        //                         oldest.info.started_at = self.current_time;
        //                     }
        //                 }

        //                 None
        //             }
        //             _ => Some(message),
        //         };

        //         // it wasn't note on or note off, so we better make sure all the voices get it
        //         if let Some(message_to_pass_to_all) = message_to_pass_to_all {
        //             for voice in self.voices.iter_mut() {
        //                 if voice.info.active {
        //                     let subgraph_input_node = voice.graph.get_node_mut(child_input_node).unwrap();
        //                     subgraph_input_node
        //                         .accept_midi_input(MidiSocketType::Default, smallvec![message_to_pass_to_all.clone()]);
        //                 }
        //             }
        //         }
        //     }
        // }
    }

    fn post_init(&mut self, init_state: NodeInitState, child_graph: Option<NodeGraphAndIo>) -> NodeResult<()> {
        // for i in 0..self.polyphony {
        //     if i as usize >= self.voices.len() {
        //         self.voices.push(Voice {
        //             graph: graph.clone(),
        //             info: PolyphonicInfo::new(self.current_time),
        //             is_first_time: true,
        //         });
        //     } else {
        //         self.voices[i as usize] = Voice {
        //             graph: graph.clone(),
        //             info: PolyphonicInfo::new(self.current_time),
        //             is_first_time: true,
        //         };
        //     }
        // }

        // if self.voices.len() > self.polyphony as usize {
        //     self.voices.truncate(self.polyphony as usize);
        // }

        // self.traverser = Traverser::get_traverser(graph).unwrap();
        // self.child_io_nodes = Some((input_node, output_node));

        NodeOk::no_warnings(())
    }

    fn process(&mut self, state: NodeProcessState, streams_in: &[f32], streams_out: &mut [f32]) -> NodeResult<()> {
        // let (child_input_node, child_output_node) = self.child_io_nodes.unwrap();

        // self.current_time = state.current_time;

        // let mut output = 0.0;

        // // loop through voices
        // for voice in self.voices.iter_mut() {
        //     if voice.info.active {
        //         // if it's active, process it
        //         self.traverser
        //             .traverse(
        //                 &mut voice.graph,
        //                 voice.is_first_time,
        //                 state.current_time,
        //                 state.script_engine,
        //                 state.global_state,
        //             )
        //             .map_err(|err| NodeError::InnerGraphErrors {
        //                 errors_and_warnings: err,
        //             })?;

        //         let subgraph_output_node = voice.graph.get_node_mut(child_output_node).unwrap();
        //         let child_output = subgraph_output_node.get_stream_output(StreamSocketType::Audio);

        //         output += child_output;

        //         if (voice.info.last_output_value - child_output).abs() < DIFFERENCE_THRESHOLD {
        //             voice.info.duration_of_same_output += 1;

        //             if voice.info.duration_of_same_output > SAME_VALUE_LENGTH_THRESHOLD {
        //                 // mark voice as inactive
        //                 voice.info.active = false;
        //                 voice.info.duration_of_same_output = 0;
        //             }
        //         } else {
        //             voice.info.duration_of_same_output = 0;
        //             voice.info.last_output_value = child_output;
        //         }

        //         voice.is_first_time = false;
        //     }
        // }

        // streams_out[0] = output;

        NodeOk::no_warnings(())
    }
}
