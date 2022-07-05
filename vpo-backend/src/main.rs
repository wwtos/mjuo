use std::error::Error;
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

use async_std::channel::{unbounded, Receiver, Sender};

use async_std::task::block_on;
use ipc::ipc_message::IPCMessage;
use node_engine::connection::{MidiSocketType, StreamSocketType, SocketType};
use node_engine::graph::{Graph, PossibleNode};

use node_engine::graph_manager::GraphManager;
use node_engine::node::NodeIndex;
use node_engine::nodes::midi_input::MidiInNode;
use node_engine::nodes::output::OutputNode;
use node_engine::nodes::variants::NodeVariant;
use node_engine::socket_registry::SocketRegistry;
use node_engine::traversal;
use node_engine::traversal::traverser::{Traverser};
use rhai::Engine;
use serde_json::json;
use sound_engine::backend::alsa_midi::AlsaMidiClientBackend;
use sound_engine::backend::MidiClientBackend;
use sound_engine::backend::{pulse::PulseClientBackend, AudioClientBackend};
use sound_engine::constants::{BUFFER_SIZE, SAMPLE_RATE};
use sound_engine::SoundConfig;

use ipc::ipc_server::IPCServer;
use sound_engine::midi::messages::MidiData;
use sound_engine::midi::parse::MidiParser;
use vpo_backend::route;

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
    graph: &mut Graph,
    to_server: &Sender<IPCMessage>,
    traverser: &mut Traverser,
    sound_config: &SoundConfig,
    socket_registry: &mut SocketRegistry,
    scripting_engine: &Engine,
) {
    let result = route(msg, graph, to_server, sound_config, socket_registry, scripting_engine);

    match result {
        Ok(route_result) => {
            let should_reindex_graph = match route_result {
                Some(route_result) => route_result.should_reindex_graph,
                None => false
            };

            if should_reindex_graph {
                *traverser = Traverser::get_traverser(graph);
            }

            // TODO: this is naive, keep track of what nodes need their defaults updated
            for (i, node) in graph.get_nodes().iter().enumerate() {
                if let PossibleNode::Some(node) = node {
                    traverser.update_node_defaults(graph, &NodeIndex {
                        index: i,
                        generation: node.generation
                    });
                }
            }
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
    let mut backend: Box<dyn AudioClientBackend> = Box::new(PulseClientBackend::new());
    backend.connect()?;

    Ok(backend)
}

fn connect_midi_backend() -> Result<Box<dyn MidiClientBackend>, Box<dyn Error>> {
    let mut backend: Box<dyn MidiClientBackend> = Box::new(AlsaMidiClientBackend::new());
    backend.connect()?;

    Ok(backend)
}

fn get_midi(
    midi_backend: &mut Box<dyn MidiClientBackend>,
    parser: &mut MidiParser,
) -> Vec<MidiData> {
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

    let mut socket_registry = SocketRegistry::new();
    SocketType::register_defaults(&mut socket_registry);

    let mut scripting_engine = Engine::new_raw();

    let mut graph_manager = GraphManager::new();
    let graph_index = graph_manager.new_graph();

    let graph = graph_manager.get_graph_mut(&graph_index).unwrap();
    let output_node = graph.add_node(NodeVariant::OutputNode(OutputNode::default()), &mut socket_registry, &scripting_engine);
    let midi_in_node = graph.add_node(NodeVariant::MidiInNode(MidiInNode::default()), &mut socket_registry, &scripting_engine);
    let mut traverser = Traverser::get_traverser(&graph);

    let backend = connect_backend()?;

    let sound_config = SoundConfig {
        sample_rate: SAMPLE_RATE,
    };

    let mut midi_backend = connect_midi_backend()?;
    let mut parser = MidiParser::new();

    let mut buffer_index = 0;
    let start = Instant::now();

    let mut is_first_time = true;

    loop {
        let msg = from_server.try_recv();

        if let Ok(msg) = msg {
            handle_msg(msg, graph, &to_server, &mut traverser, &sound_config, &mut socket_registry, &scripting_engine);
            // TODO: this shouldn't reset `is_first_time` for just any message
            is_first_time = true;
        }

        let midi = get_midi(&mut midi_backend, &mut parser);

        if !midi.is_empty() {
            let midi_node = graph.get_node(&midi_in_node).unwrap().node;
            let mut midi_node = (*midi_node).borrow_mut();

            midi_node.accept_midi_input(&MidiSocketType::Default, midi.clone());
        } else {
            let midi_node = graph.get_node(&midi_in_node).unwrap().node;
            let mut midi_node = (*midi_node).borrow_mut();

            midi_node.accept_midi_input(&MidiSocketType::Default, Vec::new());
        }

        let mut buffer = [0_f32; BUFFER_SIZE];

        for (i, sample) in buffer.iter_mut().enumerate() {
            let current_time = (buffer_index * BUFFER_SIZE + i) as i64;
            let traversal_errors = traverser.traverse(graph, is_first_time, current_time, &scripting_engine);

            if let Err(errors) = traversal_errors {
                println!("{:?}", errors);
            }

            let output_node = graph.get_node(&output_node).unwrap().node;
            let output_node = (*output_node).borrow();

            let audio = output_node.get_stream_output(&StreamSocketType::Audio);

            *sample = audio;

            is_first_time = false;
        }

        backend.write(&buffer)?;

        let now = Instant::now() - start;
        let sample_duration =
            Duration::from_secs_f64(1.0 / (SAMPLE_RATE as f64 / BUFFER_SIZE as f64));
        let buffer_time =
            Duration::from_secs_f64((buffer_index as f64) * sample_duration.as_secs_f64());

        if !(now > buffer_time || buffer_time - now < Duration::from_secs_f64(0.3)) {
            thread::sleep(sample_duration);
        }

        buffer_index += 1;
    }
}
