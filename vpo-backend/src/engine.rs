use std::sync::{Arc, RwLock};
use std::time::Instant;
use std::{collections::BTreeMap, time::Duration};
use std::{mem, thread};

use clocked::midi::{MidiData, MidiMessage};
use clocked::TimedValue;
use clocked::{
    cpal::{CpalSink, CpalSource},
    midir::{MidirSink, MidirSource},
};
use node_engine::connection::{Primitive, Socket};
use node_engine::io_routing::{DeviceDirection, DeviceType};
use node_engine::node::{NodeIndex, NodeState};
use node_engine::nodes::NodeVariant;
use node_engine::resources::Resources;
use node_engine::{
    io_routing::IoRoutes,
    node::buffered_traverser::BufferedTraverser,
    state::{FromNodeEngine, ToNodeEngine},
};
use sound_engine::SoundConfig;

pub enum ToAudioThread {
    NodeEngineUpdate(ToNodeEngine),
    NewCpalSink { name: String, sink: CpalSink },
    NewCpalSource { name: String, source: CpalSource },
    NewMidirSink { name: String, sink: MidirSink },
    NewMidirSource { name: String, source: MidirSource },
    NewRouteRules { rules: IoRoutes },
}

/// start the sound engine (run in priority thread if possible)
pub fn start_sound_engine(
    resource_lock: Arc<RwLock<Resources>>,
    msg_in: flume::Receiver<ToAudioThread>,
    msg_out: flume::Sender<FromNodeEngine>,
) {
    let mut sound_config = SoundConfig::default();
    let mut io_routing: IoRoutes = IoRoutes { rules: vec![] };

    let mut stream_sinks: BTreeMap<String, (CpalSink, Vec<f32>)> = BTreeMap::new();
    let mut stream_sources: BTreeMap<String, (CpalSource, Vec<f32>)> = BTreeMap::new();

    let mut midi_sinks: BTreeMap<String, (MidirSink, Vec<MidiMessage>)> = BTreeMap::new();
    let mut midi_sources: BTreeMap<String, (MidirSource, Vec<MidiMessage>)> = BTreeMap::new();
    let mut traverser: Option<BufferedTraverser> = None;

    let mut new_states: Vec<(NodeIndex, serde_json::Value)> = vec![];
    let mut current_graph_state: Option<BTreeMap<NodeIndex, NodeState>> = None;
    let mut new_defaults: Vec<(NodeIndex, Socket, Primitive)> = vec![];

    let start = Instant::now();
    let mut buffer_time = Duration::ZERO;

    let mut debug_counter = 0;
    let mut emitted_count = 0;

    loop {
        let sample_duration =
            Duration::from_secs_f64(sound_config.buffer_size as f64 / sound_config.sample_rate as f64);

        if let Ok(msg) = msg_in.try_recv() {
            match msg {
                ToAudioThread::NodeEngineUpdate(update) => match update {
                    ToNodeEngine::NewTraverser(new_traverser) => {
                        traverser = Some(new_traverser);
                    }
                    ToNodeEngine::NewDefaults(defaults) => {
                        new_defaults = defaults;
                    }
                    ToNodeEngine::NewNodeState(new) => {
                        new_states.extend(new.into_iter());
                    }
                    ToNodeEngine::CurrentNodeStates(current) => {
                        current_graph_state = Some(current);
                    }
                },
                ToAudioThread::NewCpalSink { name, sink } => {
                    let channels = sink.channels();
                    stream_sinks.insert(name, (sink, vec![0.0; sound_config.buffer_size * channels]));
                }
                ToAudioThread::NewCpalSource { name, source } => {
                    let channels = source.channels();
                    stream_sources.insert(name, (source, vec![0.0; sound_config.buffer_size * channels]));
                }
                ToAudioThread::NewMidirSink { name, sink } => {
                    midi_sinks.insert(name, (sink, Vec::with_capacity(128)));
                }
                ToAudioThread::NewMidirSource { name, source } => {
                    midi_sources.insert(name, (source, Vec::with_capacity(128)));
                    dbg!(&midi_sources);
                }
                ToAudioThread::NewRouteRules { rules: new_rules } => {
                    dbg!(&new_rules);
                    io_routing = new_rules;
                }
            };
        }

        // receive all incoming values and store them in buffers
        // (this allows for overlap when inputting)
        for (_, (source, buffer)) in stream_sources.iter_mut() {
            for sample in buffer.iter_mut() {
                *sample = source.interleaved_in.pop().unwrap_or(0.0);
            }
        }

        for (_, (source, buffer)) in midi_sources.iter_mut() {
            buffer.clear();

            while let Ok(data) = source.receiver.try_recv() {
                let TimedValue { since_start, value } = data;

                let data = match value {
                    // why do people use note ons for note offs??
                    MidiData::NoteOn {
                        channel,
                        note,
                        velocity: 0,
                    } => MidiData::NoteOff {
                        channel,
                        note,
                        velocity: 0,
                    },
                    _ => value,
                };

                buffer.push(MidiMessage {
                    data: data,
                    timestamp: since_start,
                });
            }
        }

        if let Some(traverser) = &mut traverser {
            // handle routing
            for rule in &io_routing.rules {
                if rule.device_direction == DeviceDirection::Source {
                    match rule.device_type {
                        DeviceType::Midi => {
                            if let Some((_, buffer)) = midi_sources.get(&rule.device_id) {
                                if !buffer.is_empty() {
                                    let node = traverser.get_node_mut(rule.node).unwrap();

                                    match node {
                                        // TODO: make sure buffer cloning isn't too expensive
                                        NodeVariant::InputsNode(inputs_node) => {
                                            inputs_node.set_midis(vec![buffer.clone()])
                                        }
                                        _ => panic!("connected node is not input node"),
                                    }
                                }
                            }
                        }
                        DeviceType::Stream => {
                            if let Some((source, buffer)) = stream_sources.get(&rule.device_id) {
                                let node = traverser.get_node_mut(rule.node).unwrap();

                                match node {
                                    NodeVariant::InputsNode(inputs_node) => {
                                        for (sample, sample_in) in inputs_node.streams_mut()[rule.node_socket]
                                            [rule.node_channel]
                                            .iter_mut()
                                            .zip(buffer.iter().skip(rule.device_channel).step_by(source.channels()))
                                        {
                                            *sample = *sample_in;
                                        }

                                        inputs_node.streams_mut()[rule.node_socket][rule.node_channel]
                                            .copy_from_slice(&buffer[..]);
                                    }
                                    _ => panic!("connected node is not input node"),
                                }
                            }
                        }
                    }
                }
            }

            let resources = resource_lock.read().unwrap();
            let updated_node_states = mem::replace(&mut new_states, vec![]);

            let result = traverser.step(&*resources, updated_node_states, current_graph_state.as_ref());
            current_graph_state = None;

            for rule in &io_routing.rules {
                if rule.device_direction == DeviceDirection::Sink {
                    match rule.device_type {
                        DeviceType::Midi => {
                            if let Some((_, buffer)) = midi_sinks.get_mut(&rule.device_id) {
                                let node = traverser.get_node_mut(rule.node).unwrap();

                                match node {
                                    NodeVariant::OutputsNode(node) => {
                                        for message in &node.get_midis()[rule.node_socket][rule.node_channel] {
                                            buffer.push(message.clone());
                                        }
                                    }
                                    _ => panic!("connected node is not output node"),
                                }
                            }
                        }
                        DeviceType::Stream => {
                            if let Some((sink, buffer)) = stream_sinks.get_mut(&rule.device_id) {
                                let node = traverser.get_node_mut(rule.node).unwrap();

                                match node {
                                    NodeVariant::OutputsNode(node) => {
                                        for (sample, out) in node.get_streams()[rule.node_socket][rule.node_channel]
                                            .iter()
                                            .zip(buffer.iter_mut().skip(rule.device_channel).step_by(sink.channels()))
                                        {
                                            *out += sample;
                                        }
                                    }
                                    _ => panic!("connected node is not output node"),
                                }
                            }
                        }
                    }
                }
            }

            for (node_index, socket, value) in &new_defaults {
                let _ = traverser.input_value_default(*node_index, socket, *value);
            }

            if result.request_for_graph_state {
                let _ = msg_out.send(FromNodeEngine::GraphStateRequested);
            }

            if !result.state_changes.is_empty() {
                let _ = msg_out.send(FromNodeEngine::NodeStateUpdates(result.state_changes));
            }

            if !result.requested_state_updates.is_empty() {
                let _ = msg_out.send(FromNodeEngine::RequestedStateUpdates(result.requested_state_updates));
            }
        }

        let mut xrun_count = 0;

        for (_, (sink, buffer)) in stream_sinks.iter_mut() {
            emitted_count += buffer.len() / sink.channels();

            for sample in buffer.iter_mut() {
                if let Err(_) = sink.interleaved_out.push(*sample) {
                    xrun_count += 1;
                }

                *sample = 0.0;
            }
        }

        for (_, (sink, buffer)) in midi_sinks.iter_mut() {
            for message in buffer.iter() {
                let _ = sink.sender.send(message.data.clone());
            }

            buffer.clear();
        }

        buffer_time += sample_duration;

        let now = Instant::now() - start;

        if buffer_time > now {
            thread::sleep(buffer_time - now);
        }

        // println!("emitted/second: {}", emitted_count as f64 / buffer_time.as_secs_f64());

        debug_counter += 1;
    }
}
