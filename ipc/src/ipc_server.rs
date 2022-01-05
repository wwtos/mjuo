use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

use crate::ipc::communication_constants::*;


pub struct IPCServer {

}

impl IPCServer {
    pub fn open() {
        let listener = TcpListener::bind("127.0.0.1:26642").unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();

            handle_connection(stream);
        }
    }
}

fn build_message(protocol: u8, data: &[u8]) -> Vec<u8> {
    // message: first byte is message type, next four bytes are data length, after that is data
    let mut message: Vec<u8> = vec![0; data.len() + 5];

    message[0] = protocol;
    
    let len = u32::to_be_bytes(data.len() as u32);
    message[1..5].clone_from_slice(&len[0..4]);

    message[5..data.len() + 5].clone_from_slice(data);

    message
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1];
    stream.read(&mut buffer).unwrap();

    let message_type = buffer[0];

    match message_type {
        PING => {
            stream.write(&[PONG]).unwrap();
        },
        PONG => {

        },
        DATA_BINARY => {
            let mut message_length_buf = [0; 4];
            stream.read_exact(&mut message_length_buf).unwrap();

            let message_length = u32::from_be_bytes(message_length_buf) as usize;

            let mut message = vec![0; message_length];
            stream.read_exact(message.as_mut_slice()).unwrap();

            println!("message: {}", String::from_utf8_lossy(message.as_slice()));
        },
        _ => {
            unreachable!("This isn't a protocol available");
        }
    }

    let response = "Hello client";

    println!("sending: {:?}", &build_message(DATA_BINARY, response.as_bytes()));
    stream.write(&build_message(DATA_BINARY, response.as_bytes())).unwrap();
    stream.flush().unwrap();
}
