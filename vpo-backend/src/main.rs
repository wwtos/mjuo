use std::error::Error;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use async_std::channel::{unbounded, Receiver, Sender};

use async_std::task::block_on;
use ipc::ipc_message::IPCMessage;
use node_engine::state::NodeEngineState;
use serde_json::json;
use sound_engine::backend::alsa_midi::AlsaMidiClientBackend;
use sound_engine::backend::pulse::PulseClientBackend;
use sound_engine::backend::AudioClientBackend;
use sound_engine::backend::MidiClientBackend;
use sound_engine::constants::{BUFFER_SIZE, SAMPLE_RATE};
use sound_engine::SoundConfig;

use ipc::ipc_server::IPCServer;
use sound_engine::midi::messages::MidiData;
use sound_engine::midi::parse::MidiParser;
use sound_engine::sampling::audio_loader::AudioLoader;
use vpo_backend::state::GlobalState;
use vpo_backend::{route, RouteReturn};

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

    // let mut output_file = std::fs::File::create("audio.raw").unwrap();

    let sound_config = SoundConfig {
        sample_rate: SAMPLE_RATE,
    };

    let mut engine_state = NodeEngineState::new(sound_config);
    let mut global_state = GlobalState::new();
    let mut audio_loader = AudioLoader::new();

    audio_loader.load(&PathBuf::from("060-C.wav"))?;

    let mut backend = connect_backend()?;

    let mut midi_backend = connect_midi_backend()?;
    let mut parser = MidiParser::new();

    let mut buffer_index = 0;
    let start = Instant::now();

    let mut is_first_time = true;

    loop {
        let msg = from_server.try_recv();

        if let Ok(msg) = msg {
            handle_msg(msg, &to_server, &mut engine_state, &mut global_state);

            // TODO: this shouldn't reset `is_first_time` for just any message
            is_first_time = true;
        }

        let midi = get_midi(&mut midi_backend, &mut parser);

        let mut buffer = [0_f32; BUFFER_SIZE];

        for (i, sample) in buffer.iter_mut().enumerate() {
            let current_time = (buffer_index * BUFFER_SIZE + i) as i64;

            let midi_to_input = if is_first_time { midi.clone() } else { Vec::new() };

            *sample = engine_state.step(current_time, is_first_time, midi_to_input);

            is_first_time = false;
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
