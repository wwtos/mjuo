use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use async_std::channel::{unbounded, Receiver, Sender};

use async_std::sync::Mutex;
use async_std::task::block_on;
use ipc::ipc_message::IPCMessage;
use node_engine::connection::{Connection, MidiSocketType, StreamSocketType, SocketDirection};
use node_engine::errors::NodeError;
use node_engine::graph::Graph;

use node_engine::graph_manager::GraphManager;
use node_engine::graph_traverse::calculate_graph_traverse_order;
use node_engine::node::NodeIndex;
use node_engine::nodes::midi_input::MidiInNode;
use node_engine::nodes::output::OutputNode;
use node_engine::nodes::variants::{new_variant, NodeVariant};
use serde_json::json;
use serde_json::Value;
use sound_engine::SoundConfig;
use sound_engine::backend::alsa_midi::AlsaMidiClientBackend;
use sound_engine::backend::MidiClientBackend;
use sound_engine::backend::{pulse::PulseClientBackend, AudioClientBackend};
use sound_engine::constants::{BUFFER_SIZE, SAMPLE_RATE};

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

fn update_graph(graph: &Graph, to_server: &Sender<IPCMessage>) {
    let json = graph.serialize().unwrap();

    block_on(async {
        to_server
            .send(IPCMessage::Json(json! {{
                "action": "graph/updateGraph",
                "payload": json
            }}))
            .await
    })
    .unwrap();
}

fn handle_msg(
    msg: IPCMessage,
    graph: &mut Graph,
    to_server: &Sender<IPCMessage>,
    traverse_order: &mut Vec<NodeIndex>,
    sound_config: &SoundConfig,
) {
    let result = route(msg, graph, to_server, sound_config);
    println!("\n\n{:?}\n\n", traverse_order);

    match result {
        Ok(route_result) => {
            let should_reindex_graph;

            if let Some(route_result) = route_result {
                should_reindex_graph = route_result.should_reindex_graph;
            } else {
                should_reindex_graph = false;
            }

            if should_reindex_graph {
                *traverse_order = calculate_graph_traverse_order(graph);
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

fn traverse_graph(graph: &mut Graph, traverse_order: &[NodeIndex], is_first_time: bool) -> Result<(), NodeError> {
    for node_index in traverse_order {
        let node_wrapper = graph.get_node(node_index).unwrap().node;
        let mut node_wrapper = (*node_wrapper).borrow_mut();

        let referenced_nodes = node_wrapper.list_connected_input_sockets();

        // TODO: This is super unoptimized
        for input_socket in node_wrapper.list_input_sockets() {
            let possible_connection = referenced_nodes.iter().find(|connection| connection.to_socket_type == input_socket);

            if let Some(connection) = possible_connection {
                let other_node_wrapper = graph.get_node(&connection.from_node).unwrap().node;
                let other_node_wrapper = (*other_node_wrapper).borrow();

                match &connection.from_socket_type {
                    node_engine::connection::SocketType::Stream(stream_type) => {
                        let sample = other_node_wrapper.get_stream_output(stream_type.clone());
                        node_wrapper.accept_stream_input(
                            connection.to_socket_type.clone().as_stream().unwrap(),
                            sample,
                        );
                    }
                    node_engine::connection::SocketType::Midi(midi_type) => {
                        let midi = other_node_wrapper.get_midi_output(midi_type.clone());

                        if !midi.is_empty() {
                            node_wrapper
                                .accept_midi_input(connection.to_socket_type.clone().as_midi().unwrap(), midi);
                        }
                    }
                    node_engine::connection::SocketType::Value(value_type) => {
                        let value = other_node_wrapper.get_value_output(value_type.clone());

                        if let Some(value) = value {
                            node_wrapper.accept_value_input(
                                connection.to_socket_type.clone().as_value().unwrap(),
                                value,
                            );
                        }
                    }
                    node_engine::connection::SocketType::NodeRef(_) => {},
                    node_engine::connection::SocketType::MethodCall(_) => todo!(),
                }
            } else {
                // find the default value for this one
                let default = node_wrapper.get_default(&input_socket, &SocketDirection::Input);

                match default {
                    node_engine::node::NodeRow::StreamInput(socket_type, default) => {
                        node_wrapper.accept_stream_input(socket_type, default);
                    },
                    node_engine::node::NodeRow::MidiInput(socket_type, default) => {
                        if is_first_time {
                            node_wrapper.accept_midi_input(socket_type, default);
                        }
                    },
                    node_engine::node::NodeRow::ValueInput(socket_type, default) => {
                        if is_first_time {
                            node_wrapper.accept_value_input(socket_type, default);
                        }
                    },
                    node_engine::node::NodeRow::NodeRefInput(_) => {},
                    _ => unreachable!()
                }
            }
        }

        node_wrapper.process();
    }

    Ok(())
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

    let mut graph_manager = GraphManager::new();
    let graph_index = graph_manager.new_graph();

    let graph = graph_manager.get_graph_mut(&graph_index).unwrap();
    let output_node = graph.add_node(NodeVariant::OutputNode(OutputNode::default()));
    let midi_in_node = graph.add_node(NodeVariant::MidiInNode(MidiInNode::default()));
    let mut traverse_order = calculate_graph_traverse_order(graph);

    let backend = connect_backend()?;

    let sound_config = SoundConfig {
        sample_rate: SAMPLE_RATE
    };

    let mut midi_backend = connect_midi_backend()?;
    let mut parser = MidiParser::new();

    let mut buffer_index = 0;
    let start = Instant::now();

    let mut is_first_time = true;

    loop {
        let msg = from_server.try_recv();

        if let Ok(msg) = msg {
            handle_msg(msg, graph, &to_server, &mut traverse_order, &sound_config);
            // TODO: this shouldn't reset `is_first_time` for just any message
            is_first_time = true;
        }

        let midi = get_midi(&mut midi_backend, &mut parser);

        if !midi.is_empty() {
            let midi_node = graph.get_node(&midi_in_node).unwrap().node;
            let mut midi_node = (*midi_node).borrow_mut();

            midi_node.accept_midi_input(MidiSocketType::Default, midi.clone());
        } else {
            let midi_node = graph.get_node(&midi_in_node).unwrap().node;
            let mut midi_node = (*midi_node).borrow_mut();

            midi_node.accept_midi_input(MidiSocketType::Default, Vec::new());
        }

        let mut buffer = [0_f32; BUFFER_SIZE];

        for sample in buffer.iter_mut() {
            traverse_graph(graph, &traverse_order, is_first_time).unwrap();

            let output_node = graph.get_node(&output_node).unwrap().node;
            let output_node = (*output_node).borrow();

            let audio = output_node.get_stream_output(StreamSocketType::Audio);

            *sample = audio;
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

        is_first_time = false;
    }
}
