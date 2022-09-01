use std::error::Error;
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

use async_std::channel::{unbounded, Receiver, Sender};

use async_std::task::block_on;
use ipc::ipc_message::IPCMessage;
use node_engine::connection::{MidiSocketType, SocketType, StreamSocketType};
use node_engine::node_graph::PossibleNode;

use node_engine::graph_manager::{GraphIndex, GraphManager, NodeGraphWrapper, GlobalNodeIndex};
use node_engine::node::NodeIndex;
use node_engine::nodes::midi_input::MidiInNode;
use node_engine::nodes::output::OutputNode;
use node_engine::nodes::variants::NodeVariant;
use node_engine::socket_registry::SocketRegistry;
use node_engine::state::StateManager;
use rhai::Engine;
use serde_json::json;
use sound_engine::backend::alsa::AlsaAudioBackend;
use sound_engine::backend::alsa_midi::AlsaMidiClientBackend;
use sound_engine::backend::pulse::PulseClientBackend;
use sound_engine::backend::AudioClientBackend;
use sound_engine::backend::MidiClientBackend;
use sound_engine::constants::{BUFFER_SIZE, SAMPLE_RATE};
use sound_engine::SoundConfig;

use ipc::ipc_server::IPCServer;
use sound_engine::midi::messages::MidiData;
use sound_engine::midi::parse::MidiParser;
use vpo_backend::{route, write_to_file, RouteReturn};

fn start_ipc() -> (Sender<IPCMessage>, Receiver<IPCMessage>) {
    let (to_server, from_main) = unbounded::<IPCMessage>();
    let (to_main, from_server) = unbounded::<IPCMessage>();

    let to_server_cloned = to_server.clone();

    thread::spawn(move || {
        IPCServer::open(to_server_cloned.clone(), from_main, to_main);
    });

    (to_server, from_server)
}

fn handle_msg(
    msg: IPCMessage,
    to_server: &Sender<IPCMessage>,
    state: &mut StateManager
) {
    let result = route(
        msg,
        to_server,
        state
    );

    match result {
        Ok(route_result) => {
            let route_result = match route_result {
                Some(route_result) => route_result,
                None => RouteReturn::default(),
            };

            if let Some(to_reindex) = route_result.graph_to_reindex {
                graph_manager.recalculate_traversal_for_graph(*to_reindex);
            }

            // TODO: also naive
            // checks if this is a subgraph (aka not root graph), and if it is, notify
            // the parent node that it was changed
            if current_graph_index != &0 {
                let parent_nodes = graph_manager.get_subgraph_parent_nodes(*current_graph_index);

                for GlobalNodeIndex { graph_index: parent_node_graph, node_index: parent_node_index } in parent_nodes {
                    let parent_node_graph = &mut graph_manager.get_graph_wrapper_mut(parent_node_graph).unwrap().graph;
                    let subgraph = &mut graph_manager.get_graph_wrapper_mut(*current_graph_index).unwrap().graph;

                    let node = parent_node_graph.get_node_mut(&parent_node_index).unwrap();
                    node.node_init_graph(subgraph);
                }
            }

            let nodes_to_update = {
                let graph = &graph_manager.get_graph_wrapper_ref(*current_graph_index).unwrap().graph;

                // TODO: this is naive, keep track of what nodes need their defaults updated
                graph
                    .get_nodes()
                    .iter()
                    .enumerate()
                    .filter_map(|(i, node)| {
                        if let PossibleNode::Some(_, generation) = node {
                            Some(NodeIndex {
                                index: i,
                                generation: *generation,
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<NodeIndex>>()
            };

            graph_manager.update_traversal_defaults(*current_graph_index, nodes_to_update);
        }
        Err(err) => {
            let err_str = err.to_string();

            block_on(async {
                to_server
                    .send(IPCMessage::Json(json! {{
                        "action": "toast/error",
                        "payload": err_str
                    }}))
                    .await
            })
            .unwrap();
        }
    }
}

fn connect_backend() -> Result<Box<dyn AudioClientBackend>, Box<dyn Error>> {
    let mut backend: Box<dyn AudioClientBackend> = Box::new(AlsaAudioBackend::new());
    backend.connect()?;

    Ok(backend)
}

fn connect_midi_backend() -> Result<Box<dyn MidiClientBackend>, Box<dyn Error>> {
    let mut backend: Box<dyn MidiClientBackend> = Box::new(AlsaMidiClientBackend::new());
    backend.connect()?;

    Ok(backend)
}

fn get_midi(midi_backend: &mut Box<dyn MidiClientBackend>, parser: &mut MidiParser) -> Vec<MidiData> {
    let midi_in = midi_backend.read().unwrap();
    let mut messages: Vec<MidiData> = Vec::new();

    if !midi_in.is_empty() {
        parser.write_all(midi_in.as_slice()).unwrap();

        while !parser.parsed.is_empty() {
            let message = parser.parsed.pop().unwrap();
            messages.push(message);
        }
    }

    messages
}

fn main() -> Result<(), Box<dyn Error>> {
    let (to_server, from_server) = start_ipc();

    let mut output_file = std::fs::File::create("audio.raw").unwrap();

    let sound_config = SoundConfig {
        sample_rate: SAMPLE_RATE,
    };

    let mut state = StateManager::new(sound_config);

    let backend = connect_backend()?;

    let mut midi_backend = connect_midi_backend()?;
    let mut parser = MidiParser::new();

    let mut buffer_index = 0;
    let start = Instant::now();

    let mut is_first_time = true;

    loop {
        let msg = from_server.try_recv();

        if let Ok(msg) = msg {
            handle_msg(msg, &to_server, &mut state);

            // TODO: this shouldn't reset `is_first_time` for just any message
            is_first_time = true;
        }

        // we can get the graph now, it won't be controlled by the message handler anymore
        let NodeGraphWrapper { graph, traverser, .. } =
            &mut *graph_manager.get_graph_wrapper_mut(root_graph_index).unwrap();

        let mut has_midi_been_inputted = false;
        let midi = get_midi(&mut midi_backend, &mut parser);

        if !midi.is_empty() {
            let midi_node = graph.get_node_mut(&midi_in_node).unwrap();

            midi_node.accept_midi_input(&MidiSocketType::Default, midi.clone());
        } else {
            let midi_node = graph.get_node_mut(&midi_in_node).unwrap();

            midi_node.accept_midi_input(&MidiSocketType::Default, Vec::new());
        }

        let mut buffer = [0_f32; BUFFER_SIZE];

        for (i, sample) in buffer.iter_mut().enumerate() {
            let current_time = (buffer_index * BUFFER_SIZE + i) as i64;
            let traversal_errors = traverser.traverse(graph, is_first_time, current_time, &scripting_engine);

            if let Err(errors) = traversal_errors {
                println!("{:?}", errors);
            }

            let output_node = graph.get_node_mut(&output_node).unwrap();

            let audio = output_node.get_stream_output(&StreamSocketType::Audio);

            *sample = audio;

            if !has_midi_been_inputted {
                let midi_node = graph.get_node_mut(&midi_in_node).unwrap();

                midi_node.accept_midi_input(&MidiSocketType::Default, Vec::new());
            }

            is_first_time = false;
            has_midi_been_inputted = true;
        }

        backend.write(&buffer)?;
        //write_to_file(&mut output_file, &buffer)?;

        let now = Instant::now() - start;
        let sample_duration = Duration::from_secs_f64(BUFFER_SIZE as f64 / SAMPLE_RATE as f64);
        let buffer_time = Duration::from_secs_f64((buffer_index as f64) * sample_duration.as_secs_f64());

        // println!("now: {:?}, now (buffer): {:?}", now, buffer_time);

        if !(now > buffer_time || buffer_time - now < sample_duration * 2) {
            thread::sleep(sample_duration);
        }

        buffer_index += 1;
    }
}
