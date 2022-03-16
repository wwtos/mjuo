use std::error::Error;
use std::thread;

use async_std::channel::{Receiver, Sender, unbounded};

use async_std::task::block_on;
use ipc::ipc_message::IPCMessage;
use node_engine::connection::Connection;
use node_engine::errors::NodeError;
use node_engine::graph::Graph;

use node_engine::node::{self, NodeIndex};
use node_engine::nodes::variants::new_variant;
use serde_json::Value;
use serde_json::json;
use sound_engine::backend::{pulse::PulseClientBackend, AudioClientBackend};

use ipc::ipc_server::IPCServer;

fn start_ipc() -> (Sender<IPCMessage>, Receiver<IPCMessage>) {
    let (to_server, from_main) = unbounded::<IPCMessage>();
    let (to_main, from_server) = unbounded::<IPCMessage>();

    let to_server_cloned = to_server.clone();

    thread::spawn(move || {
        IPCServer::open(
            to_server_cloned.clone(),
            from_main,
            to_main
        );
    });

    (to_server, from_server)
}

fn update_graph(graph: &Graph, to_server: &Sender<IPCMessage>) {
    let json = graph.serialize().unwrap();

    block_on(async {
        to_server.send(IPCMessage::Json(json! {{
            "action": "graph/updateGraph",
            "payload": json
        }})).await
    }).unwrap();
}

fn route(msg: IPCMessage, graph: &mut Graph, to_server: &Sender<IPCMessage>) -> Result<(), NodeError> {
    let IPCMessage::Json(json) = msg;
        
    if let Value::Object(message) = json {
        let action = message.get("action");

        if let Some(action) = action {
            if let Value::String(action_name) = action {
                match action_name.as_str() {
                    "graph/get" => {
                        update_graph(&graph, &to_server);
                    },
                    "graph/newNode" => {
                        let node_type_raw = message.get("payload").unwrap();

                        if let Value::String(node_type) = node_type_raw {
                            let new_node = new_variant(node_type).unwrap();

                            graph.add_node(new_node);
                        }

                        update_graph(&graph, &to_server);
                    },
                    "graph/updateNodes" => {
                        let nodes_raw = message.get("payload").unwrap();

                        if let Value::Array(nodes_to_update) = nodes_raw {
                            for node_json in nodes_to_update {
                                let index: NodeIndex = serde_json::from_value(node_json["index"].clone())?;

                                if let Some(generational_node) = graph.get_node(&index) {
                                    let mut node = generational_node.node.borrow_mut();
                                    
                                    node.apply_json(node_json)?;
                                }                                    
                            }
                        }

                        update_graph(&graph, &to_server);
                    },
                    "graph/connectNode" => {
                        if let Value::Object(_) = &message["payload"] {
                            let connection: Connection = serde_json::from_value(message["payload"].clone())?;

                            graph.connect(connection.from_node, connection.from_socket_type, connection.to_node, connection.to_socket_type)?;
                        }

                        update_graph(&graph, &to_server);
                    },
                    "graph/disconnectNode" => {
                        if let Value::Object(_) = &message["payload"] {
                            let connection: Connection = serde_json::from_value(message["payload"].clone())?;

                            graph.disconnect(connection.from_node, connection.from_socket_type, connection.to_node, connection.to_socket_type)?;
                        }

                        update_graph(&graph, &to_server);
                    },
                    _ => {}
                };
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let (to_server, from_server) = start_ipc();
    
    let mut graph = Graph::new();

    loop {
        let msg = block_on(async {
            from_server.recv().await
        });

        let msg = msg.unwrap();
        
        match &msg {
            IPCMessage::Json(json) => {
                println!("Received: {}", json);
            }
        }

        let result = route(msg, &mut graph, &to_server);

        match result {
            Ok(()) => {},
            Err(err) => {
                let err_str = err.to_string();

                block_on(async {
                    to_server.send(IPCMessage::Json(json! {{
                        "action": "toast/error",
                        "payload": err_str
                    }})).await
                }).unwrap();
            }
        }
    }
}
