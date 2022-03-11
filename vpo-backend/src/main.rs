use std::error::Error;
use std::thread;

use async_std::channel::{Receiver, Sender, unbounded};

use async_std::task::block_on;
use ipc::ipc_message::IPCMessage;
use node_engine::graph::Graph;

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

fn main() -> Result<(), Box<dyn Error>> {
    let (to_server, from_server) = start_ipc();
    
    let mut graph = Graph::new();

    loop {
        let msg = block_on(async {
            from_server.recv().await
        });
        
        println!("Received: {:?}", msg);

        let msg = msg.unwrap();

        let IPCMessage::Json(json) = msg;
            
        if let Value::Object(message) = json {
            let action = message.get("action");

            if let Some(action) = action {
                if let Value::String(action_name) = action {
                    match action_name.as_str() {
                        "graph/get" => {
                            let json = graph.serialize().unwrap();

                            block_on(async {
                                to_server.send(IPCMessage::Json(json! {{
                                    "action": "graph/updateGraph",
                                    "payload": json
                                }})).await
                            }).unwrap();
                        },
                        "graph/newNode" => {
                            let node_type_raw = message.get("payload").unwrap();

                            if let Value::String(node_type) = node_type_raw {
                                let new_node = new_variant(node_type).unwrap();

                                graph.add_node(new_node);
                            }

                            let json = graph.serialize().unwrap();

                            block_on(async {
                                to_server.send(IPCMessage::Json(json! {{
                                    "action": "graph/updateGraph",
                                    "payload": json
                                }})).await
                            }).unwrap();
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}
