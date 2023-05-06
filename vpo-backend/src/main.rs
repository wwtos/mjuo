use std::sync::Mutex;

use futures::future::join;
use futures::StreamExt;
use node_engine::global_state::GlobalState;

use node_engine::state::NodeState;
use sound_engine::SoundConfig;
use vpo_backend::io::cpal::CpalBackend;
use vpo_backend::io::file_watcher::FileWatcher;
use vpo_backend::io::load_single;
use vpo_backend::io::midir::connect_midir_backend;
use vpo_backend::{handle_msg, start_ipc};

#[tokio::main]
async fn main() {
    main_async().await;
}

async fn main_async() {
    let (to_server, mut from_server) = start_ipc().await;

    let engine_buffer_size = 64;
    let io_requested_buffer_size = 512;

    let mut global_state = GlobalState::new(SoundConfig::default());

    // start up midi and audio
    let (receiver, _midi_stream) = connect_midir_backend().unwrap();

    let mut backend = CpalBackend::new();
    let output_device = backend.get_default_output().unwrap();

    let (mut file_watcher, mut file_receiver) = FileWatcher::new().unwrap();

    let (_stream, sender, config) = backend
        .connect(
            output_device,
            global_state.resources.clone(),
            engine_buffer_size,
            io_requested_buffer_size,
            48_000,
            receiver,
        )
        .unwrap();

    println!("sample rate: {}", config.sample_rate.0);

    global_state.sound_config = SoundConfig {
        sample_rate: config.sample_rate.0,
        buffer_size: engine_buffer_size,
    };

    // set up state
    let mut node_state = NodeState::new(&global_state).unwrap();
    sender.send(node_state.get_engine(&global_state).unwrap()).unwrap();

    let global_state = Mutex::new(global_state);

    // debugging
    // let mut output_file = File::create("out.pcm").unwrap();
    join(
        async {
            loop {
                let msg = from_server.recv().await;

                if let Ok(msg) = msg {
                    handle_msg(
                        msg,
                        &to_server,
                        &mut node_state,
                        &mut global_state.lock().unwrap(),
                        &sender,
                        &mut file_watcher,
                    )
                    .await;
                }
            }
        },
        async {
            while let Some(res) = file_receiver.next().await {
                match res {
                    Ok(event) => {
                        for e in event {
                            let _ = load_single(&e.path, &mut global_state.lock().unwrap());
                        }
                    }
                    Err(e) => println!("watch error: {:?}", e),
                }
            }
        },
    )
    .await;
}
