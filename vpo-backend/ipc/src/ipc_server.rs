use std::io::{self, prelude::*, Error, ErrorKind};
use std::io::{BufReader, BufWriter};
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};

use futures;
use futures::executor::block_on;
use serde_json::Value;
use serde_json::json;

use crate::communication_constants::*;
use crate::ipc_message::IPCMessage;

#[derive(Debug)]
enum RawMessage {
    Ping,
    Pong,
    Data(Vec<u8>),
    Json(Value),
}

pub struct IPCServer {}

impl IPCServer {
    pub fn open(_main_process_rx: Receiver<IPCMessage>, _main_process_tx: Sender<IPCMessage>) {
        let server = TcpListener::bind("127.0.0.1:26642").unwrap();

        // TODO: yes, this isn't resilient, no I don't care for now
        for client in server.incoming() {
            let client = client.unwrap();

            let mut reader = BufReader::new(&client);
            let mut writer = BufWriter::new(&client);

            let res = block_on(async move {
                loop {
                    let message = handle_message(&mut reader).await?;

                    if let RawMessage::Json(message) = message {
                        println!("{}", message);
                    }

                    let response = serde_json::to_string(&json! {{
                        "foo": "bar",
                        "baz": {
                            "la": [1, 2, 3]
                        }
                    }}).unwrap();
                
                    writer
                        .write_all(&build_message(DATA_JSON, response.as_bytes()))
                        .unwrap();
                    writer.flush().unwrap();
                }

                Ok::<(), io::Error>(())
            });
        }
    }
}

pub fn build_message(protocol: u8, data: &[u8]) -> Vec<u8> {
    // message: first byte is message type, next four bytes are data length, after that is data
    let mut message: Vec<u8> = vec![0; data.len() + 5];

    message[0] = protocol;

    let len = u32::to_be_bytes(data.len() as u32);
    message[1..5].clone_from_slice(&len[0..4]);

    message[5..data.len() + 5].clone_from_slice(data);

    message
}

async fn handle_message(stream: &mut BufReader<&TcpStream>) -> Result<RawMessage, io::Error> {
    let mut buffer = [0; 1];
    stream.read_exact(&mut buffer)?;

    let message_type = buffer[0];

    match message_type {
        PING => Ok(RawMessage::Ping),
        PONG => Ok(RawMessage::Pong),
        DATA_BINARY => {
            let mut message_length_buf = [0; 4];
            stream.read_exact(&mut message_length_buf).unwrap();

            let message_length = u32::from_be_bytes(message_length_buf) as usize;

            let mut message = vec![0; message_length];
            stream.read_exact(message.as_mut_slice()).unwrap();

            Ok(RawMessage::Data(message))
        }
        DATA_JSON => {
            let mut message_length_buf = [0; 4];
            stream.read_exact(&mut message_length_buf).unwrap();

            let message_length = u32::from_be_bytes(message_length_buf) as usize;

            let mut message = vec![0; message_length];
            stream.read_exact(message.as_mut_slice()).unwrap();

            match serde_json::from_str(&String::from_utf8_lossy(&message)) {
                Ok(json) => Ok(RawMessage::Json(json)),
                Err(err) => Err(Error::from(err))
            }
        }
        _ => {
            unreachable!("This isn't a protocol available, {:?}", buffer);
        }
    }
}
