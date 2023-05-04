use std::error::Error;

use node_engine::global_state::GlobalState;

use node_engine::state::NodeState;
use sound_engine::SoundConfig;
use vpo_backend::io::cpal::CpalBackend;
use vpo_backend::io::midir::connect_midir_backend;
use vpo_backend::{handle_msg, start_ipc};

#[tokio::main]
async fn main() {
    let (to_server, mut from_server) = start_ipc().await;

    let buffer_size = 1024;

    let mut global_state = GlobalState::new(SoundConfig::default());

    // start up midi and audio
    let (receiver, _midi_stream) = connect_midir_backend().unwrap();

    let mut backend = CpalBackend::new();
    let output_device = backend.get_default_output().unwrap();

    let (_stream, sender, config) = backend
        .connect(
            output_device,
            global_state.resources.clone(),
            buffer_size,
            44_100,
            receiver,
        )
        .unwrap();

    println!("sample rate: {}", config.sample_rate.0);

    global_state.sound_config = SoundConfig {
        sample_rate: config.sample_rate.0,
        buffer_size,
    };

    // set up state
    let mut node_state = NodeState::new(&global_state).unwrap();
    sender.send(node_state.get_engine(&global_state).unwrap()).unwrap();

    // debugging
    // let mut output_file = File::create("out.pcm").unwrap();

    loop {
        let msg = from_server.recv().await;

        if let Ok(msg) = msg {
            handle_msg(msg, &to_server, &mut node_state, &mut global_state, &sender).await;
        }
    }
}
