use std::time::Duration;

use common::osc_midi::{get_channel, is_message_reset, NOTE_OFF_C, NOTE_ON_C};

use crate::{node::buffered_traverser::BufferedTraverser, nodes::prelude::*};

use super::NodeVariant;

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
    scratch: Vec<u8>,
}

impl Clone for PolyphonicNode {
    fn clone(&self) -> Self {
        PolyphonicNode {
            voices: vec![],
            polyphony: self.polyphony,
            input_node: self.input_node,
            output_node: self.output_node,
            scratch: default_osc(),
        }
    }
}

impl NodeRuntime for PolyphonicNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let mut warnings = vec![];

        self.polyphony = params.props.get_int("polyphony")?.clamp(1, 255) as u8;

        let child_graph_index = params.child_graph.expect("a child graph index to be provided");
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

        // search for any osc input node
        self.input_node = child_graph
            .nodes_data_iter()
            .find(|(_, node)| {
                node.get_node_type() == "InputsNode"
                    && node
                        .get_property("type")
                        .and_then(|x| x.as_multiple_choice())
                        .map(|x| &x == "osc")
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
        osc_store: &mut OscStore,
        _resources: &[Resource],
    ) {
        self.scratch.clear();
        let Some(input_node) = self.input_node else { return };
        let Some(output_node) = self.output_node else { return };

        let message_view = ins.osc(0)[0]
            .get_messages(osc_store)
            .and_then(|bytes| OscView::new(bytes));

        // go through all the messages and send them to all the appropriate locations
        if let Some(message_view) = message_view {
            message_view.all_messages(|_, _, message| {
                let addr = message.address();

                if addr == NOTE_OFF_C {
                    let Some((channel, note, _)) = read_osc!(message.arg_iter(), as_int, as_int, as_int) else {
                        return;
                    };

                    // look to see if there's a note on for this one, send it a turn off
                    // message if so
                    let inputs_node = self
                        .voices
                        .iter_mut()
                        .find(|voice| {
                            voice.info.active && voice.info.note == note as u8 && voice.info.channel == channel as u8
                        })
                        .and_then(|voice| voice.traverser.get_node_mut(input_node))
                        .and_then(|node| match node {
                            NodeVariant::InputsNode(inputs_node) => Some(inputs_node),
                            _ => None,
                        });

                    if let Some(inputs_node) = inputs_node {
                        write_message(inputs_node.osc_for_writing(), message);
                    }
                } else if addr == NOTE_ON_C {
                    let Some((channel, note, _)) = read_osc!(message.arg_iter(), as_int, as_int, as_int) else {
                        return;
                    };

                    // search through for a open voice

                    // first, check if there's already one on for this note
                    let already_on = self.voices.iter_mut().find(|voice| {
                        voice.info.active && voice.info.note == note as u8 && voice.info.channel == channel as u8
                    });

                    if let Some(already_on) = already_on {
                        let inputs_node = already_on
                            .traverser
                            .get_node_mut(input_node)
                            .and_then(|node| match node {
                                NodeVariant::InputsNode(inputs_node) => Some(inputs_node),
                                _ => None,
                            });

                        if let Some(inputs_node) = inputs_node {
                            write_message(inputs_node.osc_for_writing(), message);
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
                                write_message(inputs_node.osc_for_writing(), message);
                            }

                            available.info.active = true;
                            available.info.note = note as u8;
                            available.info.channel = channel as u8;
                            available.info.started_at = context.current_time;
                        } else {
                            // just pick the oldest played note
                            let oldest = self
                                .voices
                                .iter_mut()
                                .min_by_key(|x| x.info.started_at)
                                .expect("voices to have at least one element");

                            let inputs_node = oldest.traverser.get_node_mut(input_node).and_then(|node| match node {
                                NodeVariant::InputsNode(inputs_node) => Some(inputs_node),
                                _ => None,
                            });

                            // be sure to send a note off message first
                            if let Some(inputs_node) = inputs_node {
                                write_note_off(inputs_node.osc_for_writing(), channel as u8, oldest.info.note, 0);
                            }

                            oldest.info.active = true;
                            oldest.info.note = note as u8;
                            oldest.info.channel = channel as u8;
                            oldest.info.started_at = context.current_time;
                        }
                    }
                } else {
                    // is the message a midi message and does it have a channel?
                    if let Some(channel) = get_channel(message) {
                        // if so, only send it to voices with that channel
                        let applicable_voices = self.voices.iter_mut().filter(|voice| voice.info.channel == channel);

                        for voice in applicable_voices {
                            if let Some(NodeVariant::InputsNode(inputs_node)) = voice.traverser.get_node_mut(input_node)
                            {
                                write_message(inputs_node.osc_for_writing(), message);
                            }
                        }
                    } else {
                        // otherwise just send it to everybody
                        for voice in &mut self.voices {
                            if let Some(NodeVariant::InputsNode(inputs_node)) = voice.traverser.get_node_mut(input_node)
                            {
                                write_message(inputs_node.osc_for_writing(), message);
                            }
                        }
                    }
                }

                if is_message_reset(message) {
                    for voice in &mut self.voices {
                        voice.info.active = false;
                    }
                }
            });
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
                voice.traverser.step(&context.resources, vec![], None, osc_store);

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

impl Node for PolyphonicNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        PolyphonicNode {
            voices: vec![],
            scratch: default_osc(),
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
                osc_input("default", 1),
                NodeRow::Property("polyphony".to_string(), PropertyType::Integer, Property::Integer(1)),
                NodeRow::InnerGraph,
                stream_output("audio", channels),
            ],
            child_graph_io: Some(vec![
                (Socket::Simple("osc".into(), SocketType::Osc, 1), SocketDirection::Input),
                (
                    Socket::Simple("audio".into(), SocketType::Stream, channels),
                    SocketDirection::Output,
                ),
            ]),
        }
    }
}
