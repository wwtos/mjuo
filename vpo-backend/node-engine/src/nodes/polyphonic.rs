use std::time::Duration;

use crate::{node::buffered_traverser::BufferedTraverser, nodes::prelude::*};

use super::NodeVariant;

const DIFFERENCE_THRESHOLD: f32 = 0.007;

#[derive(Debug, Clone)]
struct PolyphonicInfo {
    started_at: Duration,
    active: bool,
    note: u8,
}

impl PolyphonicInfo {
    pub fn new(started_at: Duration) -> PolyphonicInfo {
        PolyphonicInfo {
            started_at,
            active: false,
            note: 255,
        }
    }
}

#[derive(Debug)]
struct Voice {
    traverser: BufferedTraverser,
    info: PolyphonicInfo,
    is_first_time: bool,
}

#[derive(Debug)]
pub struct PolyphonicNode {
    voices: Vec<Voice>,
    polyphony: u8,
    input_node: Option<NodeIndex>,
    output_node: Option<NodeIndex>,
}

impl Clone for PolyphonicNode {
    fn clone(&self) -> Self {
        PolyphonicNode {
            voices: vec![],
            polyphony: self.polyphony,
            input_node: self.input_node,
            output_node: self.output_node,
        }
    }
}

impl NodeRuntime for PolyphonicNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let mut warnings = vec![];

        if let Some(polyphony) = params.props.get("polyphony").and_then(|x| x.as_integer()) {
            self.polyphony = polyphony.clamp(1, 255) as u8;
        }

        let child_graph_index = params.child_graph.expect("a child graph to be provided");
        let child_graph = params
            .graph_manager
            .get_graph(child_graph_index)
            .expect("the child graph to exist");

        self.voices.truncate(self.polyphony as usize);
        while self.polyphony as usize > self.voices.len() {
            let (errors_and_warnings, traverser) = BufferedTraverser::new(
                params.sound_config.clone(),
                params.graph_manager,
                child_graph_index,
                params.resources,
                params.current_time,
            )?;

            if errors_and_warnings.any() {
                warnings.push(NodeWarning::InternalErrorsAndWarnings { errors_and_warnings });
            }

            self.voices.push(Voice {
                traverser,
                info: PolyphonicInfo::new(params.current_time),
                is_first_time: true,
            });
        }

        // search for any midi input node
        self.input_node = child_graph
            .nodes_data_iter()
            .find(|(_, node)| {
                node.get_node_type() == "InputsNode"
                    && node
                        .get_property("type")
                        .and_then(|x| x.as_multiple_choice())
                        .map(|x| &x == "midi")
                        .unwrap_or(false)
            })
            .map(|x| x.0);

        // search for any stream output node
        self.output_node = child_graph
            .nodes_data_iter()
            .find(|(_, node)| {
                node.get_node_type() == "OutputsNode"
                    && node
                        .get_property("type")
                        .and_then(|x| x.as_multiple_choice())
                        .map(|x| &x == "stream")
                        .unwrap_or(false)
            })
            .map(|x| x.0);

        Ok(NodeOk::new(
            InitResult {
                changed_properties: None,
                needed_resources: vec![],
            },
            warnings,
        ))
    }

    fn process<'a>(
        &mut self,
        context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        midi_store: &mut MidiStore,
        resources: &[Resource],
    ) -> NodeResult<()> {
        // if let (Some(message_id), Some(input_node), Some(output_node)) =
        //     (ins.midi(0)[0], self.input_node, self.output_node)
        // {
        //     let messages = midi_store.borrow_midi(&message_id).unwrap();

        //     // go through all the messages and send them to all the appropriate locations
        //     for message in messages {
        //         let message_to_pass_to_all = match message.data {
        //             MidiData::NoteOff { note, .. } => {
        //                 // look to see if there's a note on for this one, send it a turn off
        //                 // message if so
        //                 for voice in self.voices.iter_mut() {
        //                     if voice.info.active && voice.info.note == note {
        //                         if let NodeVariant::InputsNode(ref mut inputs_node) =
        //                             voice.traverser.get_node_mut(input_node).unwrap()
        //                         {
        //                             inputs_node.set_midis(vec![message.clone()])
        //                         }

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
        //                     // be sure to send a note off message first
        //                     if let Some(NodeVariant::InputsNode(ref mut inputs_node)) =
        //                         already_on.traverser.get_node_mut(child_input_node)
        //                     {
        //                         inputs_node.set_midis(vec![smallvec![
        //                             MidiMessage {
        //                                 data: MidiData::NoteOff {
        //                                     channel,
        //                                     note,
        //                                     velocity: 0,
        //                                 },
        //                                 timestamp: message.timestamp - 1
        //                             },
        //                             message.clone(),
        //                         ]]);
        //                     }

        //                     already_on.info.active = true;
        //                     already_on.info.note = note;
        //                     already_on.info.started_at = ctx.current_time;
        //                 } else {
        //                     // if not, check if one is available
        //                     let available = self.voices.iter_mut().find(|voice| !voice.info.active);

        //                     if let Some(available) = available {
        //                         // TODO: test code here VV

        //                         if let Some(NodeVariant::InputsNode(ref mut inputs_node)) =
        //                             available.traverser.get_node_mut(child_input_node)
        //                         {
        //                             inputs_node.set_midis(vec![smallvec![
        //                                 MidiMessage {
        //                                     data: MidiData::NoteOff {
        //                                         channel,
        //                                         note: available.info.note,
        //                                         velocity: 0,
        //                                     },
        //                                     timestamp: message.timestamp - 1
        //                                 },
        //                                 message.clone(),
        //                             ]]);
        //                         }

        //                         available.info.active = true;
        //                         available.info.note = note;
        //                         available.info.started_at = ctx.current_time;
        //                     } else {
        //                         // just pick the oldest played note
        //                         let oldest = self
        //                             .voices
        //                             .iter_mut()
        //                             .min_by(|x, y| x.info.started_at.cmp(&y.info.started_at))
        //                             .unwrap();

        //                         // be sure to send a note off message first
        //                         if let Some(NodeVariant::InputsNode(ref mut inputs_node)) =
        //                             oldest.traverser.get_node_mut(child_input_node)
        //                         {
        //                             inputs_node.set_midis(vec![smallvec![
        //                                 MidiMessage {
        //                                     data: MidiData::NoteOff {
        //                                         channel,
        //                                         note: oldest.info.note,
        //                                         velocity: 0,
        //                                     },
        //                                     timestamp: message.timestamp - 1
        //                                 },
        //                                 message.clone(),
        //                             ]]);
        //                         }

        //                         oldest.info.active = true;
        //                         oldest.info.note = note;
        //                         oldest.info.started_at = ctx.current_time;
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
        //                     if let Some(NodeVariant::InputsNode(ref mut inputs_node)) =
        //                         voice.traverser.get_node_mut(child_input_node)
        //                     {
        //                         // TODO: this `set_midis` will override previously sent messages
        //                         inputs_node.set_midis(vec![smallvec![message_to_pass_to_all.clone()]]);
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
        //                 *frame_out += child_frame_out;
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
            polyphony: 1,
            input_node: None,
            output_node: None,
        }
    }

    fn get_io(context: &NodeGetIoContext, props: SeaHashMap<String, Property>) -> NodeIo {
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
