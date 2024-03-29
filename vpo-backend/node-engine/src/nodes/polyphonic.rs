use std::time::Duration;

use crate::{node::buffered_traverser::BufferedTraverser, nodes::prelude::*};

use super::{
    util::{is_message_reset, midi_channel},
    NodeVariant,
};

const DIFFERENCE_THRESHOLD: f32 = 0.007;
const MIN_ON_TIME: Duration = Duration::from_millis(100);

#[derive(Debug, Clone)]
struct PolyphonicInfo {
    started_at: Duration,
    active: bool,
    note: u8,
    channel: u8,
}

impl PolyphonicInfo {
    pub fn new(started_at: Duration) -> PolyphonicInfo {
        PolyphonicInfo {
            started_at,
            active: false,
            note: 255,
            channel: 255,
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

        // clear all the voices in case the graph has changed
        self.voices.clear();
        while self.voices.len() < self.polyphony as usize {
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
        _resources: &[Resource],
    ) {
        if let (Some(input_node), Some(output_node)) = (self.input_node, self.output_node) {
            let messages = if let Some(messages_id) = &ins.midi(0)[0] {
                midi_store.borrow_midi(messages_id).unwrap()
            } else {
                &[]
            };

            // go through all the messages and send them to all the appropriate locations
            for message in messages {
                let message_to_pass_to_all = match message.data {
                    MidiData::NoteOff { note, channel, .. } => {
                        // look to see if there's a note on for this one, send it a turn off
                        // message if so
                        let inputs_node = self
                            .voices
                            .iter_mut()
                            .find(|voice| voice.info.active && voice.info.note == note && voice.info.channel == channel)
                            .and_then(|voice| voice.traverser.get_node_mut(input_node))
                            .and_then(|node| match node {
                                NodeVariant::InputsNode(inputs_node) => Some(inputs_node),
                                _ => None,
                            });

                        if let Some(inputs_node) = inputs_node {
                            match inputs_node.midis_mut() {
                                Some(midis) => midis.push(message.clone()),
                                None => inputs_node.set_midis(vec![message.clone()]),
                            }
                        }

                        None
                    }
                    MidiData::NoteOn { note, channel, .. } => {
                        // search through for a open voice

                        // first, check if there's already one on for this note
                        let already_on = self.voices.iter_mut().find(|voice| {
                            voice.info.active && voice.info.note == note && voice.info.channel == channel
                        });

                        if let Some(already_on) = already_on {
                            let inputs_node =
                                already_on
                                    .traverser
                                    .get_node_mut(input_node)
                                    .and_then(|node| match node {
                                        NodeVariant::InputsNode(inputs_node) => Some(inputs_node),
                                        _ => None,
                                    });

                            if let Some(inputs_node) = inputs_node {
                                match inputs_node.midis_mut() {
                                    Some(midis) => midis.push(message.clone()),
                                    None => inputs_node.set_midis(vec![message.clone()]),
                                }
                            }

                            already_on.info.started_at = context.current_time;
                        } else {
                            // if not, check if one is available
                            let available = self.voices.iter_mut().find(|voice| !voice.info.active);

                            if let Some(available) = available {
                                let inputs_node =
                                    available
                                        .traverser
                                        .get_node_mut(input_node)
                                        .and_then(|node| match node {
                                            NodeVariant::InputsNode(inputs_node) => Some(inputs_node),
                                            _ => None,
                                        });

                                if let Some(inputs_node) = inputs_node {
                                    let note_off_message = MidiMessage {
                                        data: MidiData::NoteOff {
                                            channel: available.info.channel,
                                            note: available.info.note,
                                            velocity: 0,
                                        },
                                        timestamp: message.timestamp,
                                    };

                                    match inputs_node.midis_mut() {
                                        Some(midis) => midis.extend([note_off_message, message.clone()]),
                                        None => inputs_node.set_midis(vec![note_off_message, message.clone()]),
                                    }
                                }

                                available.info.active = true;
                                available.info.note = note;
                                available.info.channel = channel;
                                available.info.started_at = context.current_time;
                            } else {
                                // just pick the oldest played note
                                let oldest = self
                                    .voices
                                    .iter_mut()
                                    .min_by_key(|x| x.info.started_at)
                                    .expect("voices to have at least one element");

                                let inputs_node =
                                    oldest.traverser.get_node_mut(input_node).and_then(|node| match node {
                                        NodeVariant::InputsNode(inputs_node) => Some(inputs_node),
                                        _ => None,
                                    });

                                // be sure to send a note off message first
                                if let Some(inputs_node) = inputs_node {
                                    let note_off_message = MidiMessage {
                                        data: MidiData::NoteOff {
                                            channel,
                                            note: oldest.info.note,
                                            velocity: 0,
                                        },
                                        timestamp: message.timestamp,
                                    };

                                    match inputs_node.midis_mut() {
                                        Some(midis) => midis.extend([note_off_message, message.clone()]),
                                        None => inputs_node.set_midis(vec![note_off_message, message.clone()]),
                                    }
                                }

                                oldest.info.active = true;
                                oldest.info.note = note;
                                oldest.info.channel = channel;
                                oldest.info.started_at = context.current_time;
                            }
                        }

                        None
                    }
                    _ => Some(message),
                };

                // it wasn't note on or note off, so we better make sure all the voices get it
                if let Some(message_to_pass_to_all) = message_to_pass_to_all {
                    for voice in self.voices.iter_mut() {
                        if voice.info.active
                            && midi_channel(&message_to_pass_to_all.data)
                                .map(|channel| voice.info.channel == channel)
                                .unwrap_or(true)
                        {
                            let inputs_node = voice.traverser.get_node_mut(input_node).and_then(|node| match node {
                                NodeVariant::InputsNode(inputs_node) => Some(inputs_node),
                                _ => None,
                            });

                            if let Some(inputs_node) = inputs_node {
                                match inputs_node.midis_mut() {
                                    Some(midis) => midis.push(message_to_pass_to_all.clone()),
                                    None => inputs_node.set_midis(vec![message_to_pass_to_all.clone()]),
                                }
                            }
                        }
                    }
                }

                if is_message_reset(&message.data) {
                    for voice in &mut self.voices {
                        voice.info.active = false;
                    }
                }
            }

            // clear output
            for channel in outs.stream(0).iter_mut() {
                for sample in channel {
                    *sample = 0.0;
                }
            }

            // loop through voices
            for voice in self.voices.iter_mut() {
                if voice.info.active {
                    // if it's active, process it
                    voice.traverser.step(&context.resources, vec![], None, midi_store);

                    let child_graph_output = voice.traverser.get_node_mut(output_node).unwrap();

                    let child_output = match child_graph_output {
                        NodeVariant::OutputsNode(output) => output.get_streams(),
                        _ => {
                            unreachable!("Node was `{child_graph_output:?}`, not `OutputsNode`!",)
                        }
                    };

                    for (channel, voice_channel) in outs.stream(0).iter_mut().zip(child_output.iter()) {
                        for (sample_out, voice_sample_out) in channel.iter_mut().zip(voice_channel.iter()) {
                            *sample_out += voice_sample_out;
                        }
                    }

                    // audio is all less than difference threshold?
                    if (context.current_time - voice.info.started_at) > MIN_ON_TIME
                        && child_output
                            .iter()
                            .all(|channel| channel.iter().all(|frame| frame.abs() < DIFFERENCE_THRESHOLD))
                    {
                        // mark voice as inactive
                        voice.info.active = false;
                    }

                    voice.is_first_time = false;
                }
            }
        }
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

    fn get_io(context: NodeGetIoContext, props: SeaHashMap<String, Property>) -> NodeIo {
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
