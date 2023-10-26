use smallvec::smallvec;

use sound_engine::midi::messages::{MidiData, MidiMessage};

use crate::nodes::prelude::*;

use super::NodeVariant;

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
    // traverser: BufferedTraverser,
    info: PolyphonicInfo,
    is_first_time: bool,
}

#[derive(Debug, Clone)]
pub struct PolyphonicNode {
    voices: Vec<Voice>,
    polyphony: u8,
    // traverser: BufferedTraverser,
    child_io_nodes: Option<(NodeIndex, NodeIndex)>,
}

impl NodeRuntime for PolyphonicNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let mut warnings = vec![];

        if let Some(Property::Integer(polyphony)) = params.props.get("polyphony") {
            self.polyphony = (*polyphony).clamp(1, 255) as u8;
        }

        // if let Some(graph_and_io) = params.child_graph {
        //     self.voices.truncate(self.polyphony as usize);

        //     while self.polyphony as usize > self.voices.len() {
        //         let (traverser, errors_and_warnings) = BufferedTraverser::new(
        //             graph_and_io.graph_index,
        //             params.graph_manager,
        //             params.script_engine,
        //             params.resources,
        //             params.current_time,
        //             params.sound_config.clone(),
        //         )?;

        //         if errors_and_warnings.any() {
        //             warnings.push(NodeWarning::InternalErrorsAndWarnings { errors_and_warnings });
        //         }

        //         self.voices.push(Voice {
        //             traverser,
        //             info: PolyphonicInfo::new(params.current_time),
        //             is_first_time: true,
        //         });
        //     }

        //     self.child_io_nodes = Some((graph_and_io.input_index, graph_and_io.output_index));
        // }

        Ok(NodeOk::new(
            InitResult {
                changed_properties: None,
                needed_resources: vec![],
            },
            warnings,
        ))
    }

    fn process<'a, 'arena: 'a, 'brand>(
        &mut self,
        context: NodeProcessContext,
        ins: Ins<'a, 'arena, 'brand>,
        outs: Outs<'a, 'arena, 'brand>,
        token: &mut GhostToken<'brand>,
        arena: &'arena BuddyArena,
        resources: &[&Resource],
    ) -> NodeResult<()> {
        // if !ins.midis[0][0].borrow(token).is_empty() {
        //     let (child_input_node, _) = self.child_io_nodes.expect("child graph to be supplied");

        //     // have we created any voices?
        //     if !self.voices.is_empty() {
        //         // go through all the messages and send them to all the appropriate locations
        //         for message in ins.midis[0][0].borrow(token) {
        //             let message_to_pass_to_all = match message.data {
        //                 MidiData::NoteOff { note, .. } => {
        //                     // look to see if there's a note on for this one, send it a turn off
        //                     // message if so
        //                     for voice in self.voices.iter_mut() {
        //                         if voice.info.active && voice.info.note == note {
        //                             if let Some(NodeVariant::InputsNode(ref mut inputs_node)) =
        //                                 voice.traverser.get_node_mut(child_input_node)
        //                             {
        //                                 inputs_node.set_midis(vec![smallvec![message.clone()]])
        //                             }

        //                             voice.info.active = true;
        //                             voice.info.note = note;
        //                             break;
        //                         }
        //                     }

        //                     None
        //                 }
        //                 MidiData::NoteOn { note, channel, .. } => {
        //                     // search through for a open voice

        //                     // first, check if there's already one on for this note
        //                     let already_on = self
        //                         .voices
        //                         .iter_mut()
        //                         .find(|voice| voice.info.active && voice.info.note == note);

        //                     if let Some(already_on) = already_on {
        //                         // be sure to send a note off message first
        //                         if let Some(NodeVariant::InputsNode(ref mut inputs_node)) =
        //                             already_on.traverser.get_node_mut(child_input_node)
        //                         {
        //                             inputs_node.set_midis(vec![smallvec![
        //                                 MidiMessage {
        //                                     data: MidiData::NoteOff {
        //                                         channel,
        //                                         note,
        //                                         velocity: 0,
        //                                     },
        //                                     timestamp: message.timestamp - 1
        //                                 },
        //                                 message.clone(),
        //                             ]]);
        //                         }

        //                         already_on.info.active = true;
        //                         already_on.info.note = note;
        //                         already_on.info.started_at = ctx.current_time;
        //                     } else {
        //                         // if not, check if one is available
        //                         let available = self.voices.iter_mut().find(|voice| !voice.info.active);

        //                         if let Some(available) = available {
        //                             // TODO: test code here VV

        //                             if let Some(NodeVariant::InputsNode(ref mut inputs_node)) =
        //                                 available.traverser.get_node_mut(child_input_node)
        //                             {
        //                                 inputs_node.set_midis(vec![smallvec![
        //                                     MidiMessage {
        //                                         data: MidiData::NoteOff {
        //                                             channel,
        //                                             note: available.info.note,
        //                                             velocity: 0,
        //                                         },
        //                                         timestamp: message.timestamp - 1
        //                                     },
        //                                     message.clone(),
        //                                 ]]);
        //                             }

        //                             available.info.active = true;
        //                             available.info.note = note;
        //                             available.info.started_at = ctx.current_time;
        //                         } else {
        //                             // just pick the oldest played note
        //                             let oldest = self
        //                                 .voices
        //                                 .iter_mut()
        //                                 .min_by(|x, y| x.info.started_at.cmp(&y.info.started_at))
        //                                 .unwrap();

        //                             // be sure to send a note off message first
        //                             if let Some(NodeVariant::InputsNode(ref mut inputs_node)) =
        //                                 oldest.traverser.get_node_mut(child_input_node)
        //                             {
        //                                 inputs_node.set_midis(vec![smallvec![
        //                                     MidiMessage {
        //                                         data: MidiData::NoteOff {
        //                                             channel,
        //                                             note: oldest.info.note,
        //                                             velocity: 0,
        //                                         },
        //                                         timestamp: message.timestamp - 1
        //                                     },
        //                                     message.clone(),
        //                                 ]]);
        //                             }

        //                             oldest.info.active = true;
        //                             oldest.info.note = note;
        //                             oldest.info.started_at = ctx.current_time;
        //                         }
        //                     }

        //                     None
        //                 }
        //                 _ => Some(message),
        //             };

        //             // it wasn't note on or note off, so we better make sure all the voices get it
        //             if let Some(message_to_pass_to_all) = message_to_pass_to_all {
        //                 for voice in self.voices.iter_mut() {
        //                     if voice.info.active {
        //                         if let Some(NodeVariant::InputsNode(ref mut inputs_node)) =
        //                             voice.traverser.get_node_mut(child_input_node)
        //                         {
        //                             // TODO: this `set_midis` will override previously sent messages
        //                             inputs_node.set_midis(vec![smallvec![message_to_pass_to_all.clone()]]);
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // }

        // let (_, child_output_node) = self.child_io_nodes.expect("to have child graph IOs");

        // // loop through voices
        // for voice in self.voices.iter_mut() {
        //     if voice.info.active {
        //         // if it's active, process it
        //         self.traverser
        //             .traverse(ctx.current_time, ctx.script_engine, ctx.resources, vec![], None);

        //         let subgraph_output_node = voice.traverser.get_node_mut(child_output_node).unwrap();

        //         let child_output = match subgraph_output_node {
        //             NodeVariant::OutputsNode(output) => output.get_streams(),
        //             _ => {
        //                 unreachable!("Node at {child_output_node:?} was `{subgraph_output_node:?}`, not `OutputsNode`!",)
        //             }
        //         };

        //         for (channel_out, child_channel_out) in outs.streams[0].iter_mut().zip(&child_output[0]) {
        //             for (frame_out, child_frame_out) in channel_out.iter_mut().zip(child_channel_out.iter()) {
        //                 *frame_out.borrow_mut(token) += child_frame_out;
        //             }
        //         }

        //         // audio is all less than difference threshold?
        //         if child_output[0]
        //             .iter()
        //             .all(|channel| channel.iter().all(|frame| frame.abs() < DIFFERENCE_THRESHOLD))
        //         {
        //             // mark voice as inactive
        //             voice.info.active = false;
        //         }

        //         voice.is_first_time = false;
        //     }
        // }

        ProcessResult::nothing()
    }
}

impl Node for PolyphonicNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        PolyphonicNode {
            voices: vec![],
            // traverser: BufferedTraverser::default(),
            polyphony: 1,
            child_io_nodes: None,
        }
    }

    fn get_io(context: &NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
        let channels = default_channels(&props, context.default_channel_count);

        NodeIo {
            node_rows: vec![
                with_channels(context.default_channel_count),
                midi_input("default", 1),
                NodeRow::Property("polyphony".to_string(), PropertyType::Integer, Property::Integer(1)),
                NodeRow::InnerGraph,
                stream_output("audio", channels),
            ],
            child_graph_io: Some(vec![
                (
                    Socket::Simple("midi".into(), SocketType::Midi, 1),
                    SocketDirection::Input,
                ),
                (
                    Socket::Simple("audio".into(), SocketType::Stream, channels),
                    SocketDirection::Output,
                ),
            ]),
        }
    }
}
