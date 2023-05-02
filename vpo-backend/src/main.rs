use std::error::Error;
use std::thread;
use std::time::Duration;

use node_engine::global_state::GlobalState;

use node_engine::state::NodeState;
use sound_engine::midi::parse::MidiParser;
use sound_engine::SoundConfig;
use vpo_backend::io::cpal::CpalBackend;
use vpo_backend::{handle_msg, start_ipc, write_to_file};

fn main() -> Result<(), Box<dyn Error>> {
    let (to_server, from_server) = start_ipc();

    let mut global_state = GlobalState::new(SoundConfig::default());

    // start up audio/midi
    let mut backend = CpalBackend::new();
    let output_device = backend.get_default_output().unwrap();

    let (_stream, sender, config) = backend
        .connect(output_device, global_state.resources.clone(), 128)
        .unwrap();

    global_state.sound_config = SoundConfig {
        sample_rate: config.sample_rate.0,
        buffer_size: 128,
    };

    // let mut midi_backend = connect_midi_backend()?;
    let mut midi_parser = MidiParser::new();

    // set up state
    let mut node_state = NodeState::new(&global_state).unwrap();
    sender.send(node_state.get_engine(&global_state).unwrap()).unwrap();

    // debugging
    // let mut output_file = File::create("out.pcm").unwrap();

    loop {
        let msg = from_server.recv_blocking();

        if let Ok(msg) = msg {
            handle_msg(msg, &to_server, &mut node_state, &mut global_state);
        }
    }
}
