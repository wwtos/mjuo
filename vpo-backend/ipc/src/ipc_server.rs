use async_std::channel::{Sender, Receiver};
use async_std::io::{self, WriteExt};
use async_std::io::Error;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;

use futures::{self, AsyncReadExt, try_join};
use futures::executor::block_on;
use serde_json::{Value};

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
    pub fn open(
        _to_server: Sender<IPCMessage>,
        from_main: Receiver<IPCMessage>,
        to_main: Sender<IPCMessage>
    ) {
        block_on(async {
            let listener = TcpListener::bind("127.0.0.1:26642").await?;
            let mut incoming = listener.incoming();

            // TODO: yes, this isn't resilient, no I don't care for now
            while let Some(stream) = incoming.next().await {
                let stream = stream?;
                let (mut reader, mut writer) = &mut (&stream, &stream);

                // try_join ensures if either one errors out, the loop will continue
                let _ = try_join!(async {
                    loop {
                        let message = handle_message(&mut reader).await?;

                        if let RawMessage::Json(message) = &message {
                            println!("{}", message);
                        }

                        match message {
                            RawMessage::Json(json) => {
                                to_main.send(IPCMessage::Json(json)).await?;
                            },
                            _ => {}
                        }
                    }

                    #[allow(unreachable_code)]
                    Ok::<(), crate::error::IPCError>(())
                }, async {
                    loop {
                        let message = from_main.recv().await?;

                        match message {
                            IPCMessage::Json(json) => {
                                let message = build_message_json(json);

                                WriteExt::write_all(&mut writer, &message).await?;
                            }
                        }
                    }

                    #[allow(unreachable_code)]
                    Ok::<(), crate::error::IPCError>(())
                });
            }

            Ok::<(), crate::error::IPCError>(())
        }).unwrap();
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

pub fn build_message_json(json: Value) -> Vec<u8> {
    build_message(DATA_JSON, json.to_string().as_bytes())
}

async fn handle_message(stream: &mut &TcpStream) -> Result<RawMessage, io::Error> {
    // read first byte for message type
    let mut buffer = [0; 1];

    AsyncReadExt::read_exact(stream, &mut buffer).await?;

    let message_type = buffer[0];

    match message_type {
        PING => Ok(RawMessage::Ping),
        PONG => Ok(RawMessage::Pong),
        DATA_BINARY => {
            // read length of message
            let mut message_length_buf = [0; 4];
            AsyncReadExt::read_exact(stream, &mut message_length_buf).await?;

            let message_length = u32::from_be_bytes(message_length_buf) as usize;

            // read message and convert to appropriate data type
            let mut message = vec![0; message_length];
            AsyncReadExt::read_exact(stream, message.as_mut_slice()).await?;

            Ok(RawMessage::Data(message))
        }
        DATA_JSON => {
            // read length of message
            let mut message_length_buf = [0; 4];
            AsyncReadExt::read_exact(stream, &mut message_length_buf).await?;

            let message_length = u32::from_be_bytes(message_length_buf) as usize;

            // read message and convert to appropriate data type
            let mut message = vec![0; message_length];
            AsyncReadExt::read_exact(stream, message.as_mut_slice()).await?;

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
