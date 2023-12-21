use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Instant;
use std::{collections::BTreeMap, time::Duration};

use clocked::cpal::{start_cpal_sink, start_cpal_source};
use clocked::midi::MidiMessage;
use clocked::{
    cpal::{CpalSink, CpalSource},
    midir::{MidirSink, MidirSource},
};
use cpal::{SampleFormat, StreamConfig};
use node_engine::io_routing::{DeviceDirection, DeviceType};
use node_engine::node::{NodeIndex, NodeState};
use node_engine::nodes::NodeVariant;
use node_engine::resources::Resources;
use node_engine::{
    io_routing::IoRoutes,
    state::{FromNodeEngine, ToNodeEngine},
    traversal::buffered_traverser::BufferedTraverser,
};
use sound_engine::SoundConfig;

use crate::io::clocked::DeviceManager;

pub enum ToAudioThread {
    NodeEngineUpdate(ToNodeEngine),
    CreateCpalSink {
        name: String,
        config: StreamConfig,
        buffer_size: usize,
        periods: usize,
    },
    CreateCpalSource {
        name: String,
        config: StreamConfig,
        buffer_size: usize,
        periods: usize,
    },
    NewMidirSink {
        name: String,
    },
    NewMidirSource {
        name: String,
    },
}

/// start the sound engine (run in priority thread if possible)
pub fn start_sound_engine(
    resource_lock: Arc<RwLock<Resources>>,
    msg_in: flume::Receiver<ToAudioThread>,
    msg_out: flume::Sender<FromNodeEngine>,
) {
    let mut device_manager = DeviceManager::new();

    let mut sound_config = SoundConfig::default();
    let mut io_routing: IoRoutes = IoRoutes { rules: vec![] };

    let mut stream_sinks: BTreeMap<String, (CpalSink, Vec<f32>)> = BTreeMap::new();
    let mut stream_sources: BTreeMap<String, (CpalSource, Vec<f32>)> = BTreeMap::new();

    let mut midi_sinks: BTreeMap<String, (MidirSink, Vec<MidiMessage>)> = BTreeMap::new();
    let mut midi_sources: BTreeMap<String, (MidirSource, Vec<MidiMessage>)> = BTreeMap::new();
    let mut traverser: Option<BufferedTraverser> = None;

    let mut new_states: Vec<(NodeIndex, serde_json::Value)> = vec![];
    let mut current_graph_state: Option<BTreeMap<NodeIndex, NodeState>> = None;

    let start = Instant::now();
    let mut buffer_time = Duration::ZERO;

    loop {
        let sample_duration =
            Duration::from_secs_f64(sound_config.buffer_size as f64 / sound_config.sample_rate as f64);

        if let Ok(msg) = msg_in.try_recv() {
            match msg {
                ToAudioThread::NodeEngineUpdate(update) => match update {
                    ToNodeEngine::NewTraverser(new_traverser) => {
                        traverser = Some(new_traverser);
                    }
                    ToNodeEngine::NewDefaults(_) => todo!(),
                    ToNodeEngine::NewNodeState(_) => todo!(),
                    ToNodeEngine::CurrentNodeStates(_) => todo!(),
                },
                ToAudioThread::CreateCpalSink {
                    name,
                    config,
                    buffer_size,
                    periods,
                } => {
                    device_manager.rescan_devices();

                    let device = device_manager
                        .take_device(
                            device_manager
                                .cpal_device_by_name(&name, DeviceDirection::Sink)
                                .unwrap(),
                        )
                        .unwrap();

                    let sink = start_cpal_sink(device, &config, SampleFormat::F32, buffer_size, periods).unwrap();

                    let channels = config.channels as usize;
                    stream_sinks.insert(name, (sink, vec![0.0; buffer_size * channels]));
                }
                ToAudioThread::CreateCpalSource {
                    name,
                    config,
                    buffer_size,
                    periods,
                } => {
                    device_manager.rescan_devices();

                    let device = device_manager
                        .take_device(
                            device_manager
                                .cpal_device_by_name(&name, DeviceDirection::Source)
                                .unwrap(),
                        )
                        .unwrap();

                    let source = start_cpal_source(device, &config, SampleFormat::F32, buffer_size, periods).unwrap();

                    let channels = config.channels as usize;
                    stream_sources.insert(name, (source, vec![0.0; sound_config.buffer_size * channels]));
                }
                ToAudioThread::NewMidirSink { name } => {
                    // device_manager.rescan_devices();

                    // midi_sinks.insert(name, (device, Vec::with_capacity(128)));
                }
                ToAudioThread::NewMidirSource { name } => {
                    // device_manager.rescan_devices();

                    // midi_sources.insert(name, (device, Vec::with_capacity(128)));
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
                buffer.push(MidiMessage {
                    data: data.value,
                    timestamp: data.since_start,
                });
            }
        }

        if let Some(traverser) = &mut traverser {
            // handle routing
            for route_rule in &io_routing.rules {
                if route_rule.device_direction == DeviceDirection::Source {
                    match route_rule.device_type {
                        DeviceType::Midi => {
                            if let Some((_, buffer)) = midi_sources.get(&route_rule.device_id) {
                                let node = traverser.get_node_mut(route_rule.node).unwrap();

                                match node {
                                    // TODO: make sure buffer cloning isn't too expensive
                                    NodeVariant::InputsNode(inputs_node) => inputs_node.set_midis(vec![buffer.clone()]),
                                    _ => panic!("connected node is not input node"),
                                }
                            }
                        }
                        DeviceType::Stream => {
                            if let Some((_, buffer)) = stream_sources.get(&route_rule.device_id) {
                                let node = traverser.get_node_mut(route_rule.node).unwrap();

                                match node {
                                    NodeVariant::InputsNode(inputs_node) => {
                                        inputs_node.streams_mut()[route_rule.node_socket][route_rule.node_channel]
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
            traverser.step(&*resources, new_states.clone(), current_graph_state.as_ref());

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
                            if let Some((_, buffer)) = stream_sinks.get_mut(&rule.device_id) {
                                let node = traverser.get_node_mut(rule.node).unwrap();

                                match node {
                                    NodeVariant::OutputsNode(node) => {
                                        for (sample, out) in node.get_streams()[rule.node_socket][rule.node_channel]
                                            .iter()
                                            .zip(buffer.iter_mut())
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
        }

        for (_, (sink, buffer)) in stream_sinks.iter_mut() {
            for sample in buffer.iter_mut() {
                let _ = sink.interleaved_out.push(*sample);
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
    }
}
