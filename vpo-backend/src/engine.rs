use std::thread;
use std::time::Instant;
use std::{collections::BTreeMap, time::Duration};

use clocked::midi::MidiMessage;
use clocked::{
    cpal::{CpalSink, CpalSource},
    midir::{MidirSink, MidirSource},
};
use node_engine::io_routing::{DeviceDirection, DeviceType};
use node_engine::nodes::NodeVariant;
use node_engine::{
    io_routing::IoRoutes,
    state::{FromNodeEngine, NodeEngineUpdate},
    traversal::buffered_traverser::BufferedTraverser,
};
use sound_engine::SoundConfig;

/// start the realtime-safe sound engine (run in priority thread if possible)
pub fn start_sound_engine(msg_in: flume::Receiver<NodeEngineUpdate>, msg_out: flume::Sender<FromNodeEngine>) {
    let current_time: i64 = 0;
    let sound_config = SoundConfig::default();
    let io_routing: IoRoutes = IoRoutes { rules: vec![] };
    let stream_sinks: BTreeMap<String, CpalSink> = BTreeMap::new();
    let stream_sources: BTreeMap<String, CpalSource> = BTreeMap::new();
    let midi_sinks: BTreeMap<String, MidirSink> = BTreeMap::new();
    let midi_sources: BTreeMap<String, MidirSource> = BTreeMap::new();
    let mut traverser: Option<BufferedTraverser> = None;

    let start = Instant::now();
    let buffer_time = Duration::ZERO;

    let sample_duration = Duration::from_secs_f64(sound_config.buffer_size as f64 / sound_config.sample_rate as f64);

    loop {
        let now = Instant::now() - start;

        if let Some(traverser) = &mut traverser {
            // handle routing
            for route_rule in &io_routing.rules {
                if route_rule.device_direction == DeviceDirection::Source {
                    match route_rule.device_type {
                        DeviceType::Midi => {
                            // TODO: fix packet jittering

                            // collect messages
                            let mut messages = vec![];
                            if let Some(midi_source) = midi_sources.get(&route_rule.device_id) {
                                while let Ok(data) = midi_source.receiver.try_recv() {
                                    messages.push(MidiMessage {
                                        data: data.value,
                                        timestamp: data.since_start,
                                    });
                                }
                            }

                            let node = traverser.get_node_mut(route_rule.node).unwrap();

                            match node {
                                NodeVariant::InputsNode(inputs_node) => inputs_node.set_midis(vec![messages]),
                                _ => panic!("connected node is not input node"),
                            }
                        }
                        DeviceType::Stream => todo!(),
                    }
                }
            }
        }

        if !(now > buffer_time || buffer_time - now < sample_duration) {
            thread::sleep(sample_duration);
        }
    }
}
