use std::convert::Infallible;
use std::error::Error;
use std::io::Write;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use async_std::channel::{unbounded, Receiver, Sender};

use async_std::sync::Mutex;
use async_std::task::block_on;
use ipc::error::IPCError;
use ipc::ipc_message::IPCMessage;
use node_engine::connection::{Connection, MidiSocketType, StreamSocketType};
use node_engine::errors::NodeError;
use node_engine::graph::Graph;

use node_engine::graph_manager::GraphManager;
use node_engine::graph_traverse::calculate_graph_traverse_order;
use node_engine::node::NodeIndex;
use node_engine::nodes::midi_input::MidiInNode;
use node_engine::nodes::output::OutputNode;
use node_engine::nodes::variants::{new_variant, NodeVariant};
use routerify::Router;
use serde_json::json;
use serde_json::Value;
use sound_engine::backend::alsa_midi::AlsaMidiClientBackend;
use sound_engine::backend::MidiClientBackend;
use sound_engine::backend::{pulse::PulseClientBackend, AudioClientBackend};
use sound_engine::constants::{BUFFER_SIZE, SAMPLE_RATE};

use hyper::{Body, Request, Response, Server, StatusCode};

use ipc::ipc_server::IPCServer;
use sound_engine::midi::messages::MidiData;
use sound_engine::midi::parse::MidiParser;

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

fn route(
    msg: IPCMessage,
    graph: &mut Graph,
    to_server: &Sender<IPCMessage>,
) -> Result<bool, NodeError> {
    let IPCMessage::Json(json) = msg;

    if let Value::Object(message) = json {
        let action = message.get("action");

        if let Some(Value::String(action_name)) = action {
            return match action_name.as_str() {
                "graph/get" => {
                    update_graph(graph, to_server);

                    Ok(false)
                }
                "graph/newNode" => {
                    let node_type_raw = message.get("payload").unwrap();

                    if let Value::String(node_type) = node_type_raw {
                        let new_node = new_variant(node_type).unwrap();

                        graph.add_node(new_node);
                    }

                    update_graph(graph, to_server);

                    Ok(true)
                }
                "graph/updateNodes" => {
                    let nodes_raw = message.get("payload").unwrap();

                    if let Value::Array(nodes_to_update) = nodes_raw {
                        for node_json in nodes_to_update {
                            let index: NodeIndex =
                                serde_json::from_value(node_json["index"].clone())?;

                            let did_apply_json =
                                if let Some(generational_node) = graph.get_node(&index) {
                                    let mut node = (*generational_node.node).borrow_mut();

                                    node.apply_json(node_json)?;

                                    true
                                } else {
                                    false
                                };

                            if did_apply_json {
                                graph.init_node(&index)?;
                            }
                        }
                    }

                    //update_graph(graph, to_server);

                    Ok(false)
                }
                "graph/connectNode" => {
                    if let Value::Object(_) = &message["payload"] {
                        let connection: Connection =
                            serde_json::from_value(message["payload"].clone())?;

                        graph.connect(
                            connection.from_node,
                            connection.from_socket_type,
                            connection.to_node,
                            connection.to_socket_type,
                        )?;
                    }

                    update_graph(graph, to_server);

                    Ok(true)
                }
                "graph/disconnectNode" => {
                    if let Value::Object(_) = &message["payload"] {
                        let connection: Connection =
                            serde_json::from_value(message["payload"].clone())?;

                        graph.disconnect(
                            connection.from_node,
                            connection.from_socket_type,
                            connection.to_node,
                            connection.to_socket_type,
                        )?;
                    }

                    update_graph(graph, to_server);

                    Ok(true)
                }
                _ => Ok(false),
            };
        }
    }

    Ok(false)
}

fn handle_msg(
    msg: IPCMessage,
    graph: &mut Graph,
    to_server: &Sender<IPCMessage>,
    traverse_order: &mut Vec<NodeIndex>,
) {
    let result = route(msg, graph, to_server);
    println!("\n\n{:?}\n\n", traverse_order);

    match result {
        Ok(graph_structure_changed) => {
            if graph_structure_changed {
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

fn traverse_graph(graph: &mut Graph, traverse_order: &[NodeIndex]) -> Result<(), NodeError> {
    for node_index in traverse_order {
        let node_wrapper = graph.get_node(node_index).unwrap().node;
        let mut node_wrapper = (*node_wrapper).borrow_mut();

        let referenced_nodes = node_wrapper.list_input_sockets();

        for connection in referenced_nodes {
            let other_node_wrapper = graph.get_node(&connection.from_node).unwrap().node;
            let other_node_wrapper = (*other_node_wrapper).borrow();

            match connection.from_socket_type {
                node_engine::connection::SocketType::Stream(stream_type) => {
                    let sample = other_node_wrapper.get_stream_output(stream_type);
                    node_wrapper.accept_stream_input(
                        connection.to_socket_type.as_stream().unwrap(),
                        sample,
                    );
                }
                node_engine::connection::SocketType::Midi(midi_type) => {
                    let midi = other_node_wrapper.get_midi_output(midi_type);

                    if !midi.is_empty() {
                        node_wrapper
                            .accept_midi_input(connection.to_socket_type.as_midi().unwrap(), midi);
                    }
                }
                node_engine::connection::SocketType::Value(value_type) => {
                    let midi = other_node_wrapper.get_value_output(value_type);

                    if let Some(midi) = midi {
                        node_wrapper.accept_value_input(
                            connection.to_socket_type.as_value().unwrap(),
                            midi,
                        );
                    }
                }
                node_engine::connection::SocketType::MethodCall(_) => todo!(),
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

struct State {
    graph_manager: Arc<Mutex<GraphManager>>,
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

    let mut midi_backend = connect_midi_backend()?;
    let mut parser = MidiParser::new();

    let mut buffer_index = 0;
    let start = Instant::now();

    loop {
        let msg = from_server.try_recv();

        if let Ok(msg) = msg {
            handle_msg(msg, graph, &to_server, &mut traverse_order);
        }

        let midi = get_midi(&mut midi_backend, &mut parser);

        if !midi.is_empty() {
            let midi_node = graph.get_node(&midi_in_node).unwrap().node;
            let mut midi_node = (*midi_node).borrow_mut();

            midi_node.accept_midi_input(MidiSocketType::Default, midi.clone());
        }

        let mut buffer = [0_f32; BUFFER_SIZE];

        for sample in buffer.iter_mut() {
            traverse_graph(graph, &traverse_order).unwrap();

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
    }
}
