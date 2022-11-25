use std::{error::Error, io::Write, thread};

use async_std::{
    channel::{unbounded, Receiver, Sender},
    task::block_on,
};
use ipc::{ipc_message::IPCMessage, ipc_server::IPCServer};
use node_engine::{global_state::GlobalState, state::NodeEngineState};
use routes::{route, RouteReturn};
use serde_json::json;
use sound_engine::{
    backend::{
        alsa::AlsaAudioBackend, alsa_midi::AlsaMidiClientBackend, pulse::PulseClientBackend, AudioClientBackend,
        MidiClientBackend,
    },
    constants::BUFFER_SIZE,
    midi::{messages::MidiData, parse::MidiParser},
};

pub mod io;
pub mod migrations;
pub mod routes;
pub mod util;

pub fn start_ipc() -> (Sender<IPCMessage>, Receiver<IPCMessage>) {
    let (to_server, from_main) = unbounded::<IPCMessage>();
    let (to_main, from_server) = unbounded::<IPCMessage>();

    let to_server_cloned = to_server.clone();

    thread::spawn(move || {
        IPCServer::open(to_server_cloned.clone(), from_main, to_main);
    });

    (to_server, from_server)
}

pub fn handle_msg(
    msg: IPCMessage,
    to_server: &Sender<IPCMessage>,
    state: &mut NodeEngineState,
    global_state: &mut GlobalState,
) {
    let result = route(msg, to_server, state, global_state);

    match result {
        Ok(route_result) => {
            match route_result {
                Some(route_result) => route_result,
                None => RouteReturn::default(),
            };
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

pub fn connect_backend() -> Result<Box<dyn AudioClientBackend>, Box<dyn Error>> {
    let mut backend: Box<dyn AudioClientBackend> = Box::new(PulseClientBackend::new());
    backend.connect()?;

    Ok(backend)
}

pub fn connect_midi_backend() -> Result<Box<dyn MidiClientBackend>, Box<dyn Error>> {
    let mut backend: Box<dyn MidiClientBackend> = Box::new(AlsaMidiClientBackend::new());
    backend.connect()?;

    Ok(backend)
}

pub fn get_midi(midi_backend: &mut Box<dyn MidiClientBackend>, parser: &mut MidiParser) -> Vec<MidiData> {
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

pub fn write_to_file(output_file: &mut std::fs::File, data: &[f32]) -> Result<(), Box<dyn Error>> {
    let mut data_out = [0_u8; BUFFER_SIZE * 4];

    // TODO: would memcpy work here faster?
    for i in 0..BUFFER_SIZE {
        let num = (data[i] as f32).to_le_bytes();

        data_out[i * 4] = num[0];
        data_out[i * 4 + 1] = num[1];
        data_out[i * 4 + 2] = num[2];
        data_out[i * 4 + 3] = num[3];
    }

    output_file.write_all(&data_out)?;

    Ok(())
}
