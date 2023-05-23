use std::sync::mpsc;

use futures::executor::block_on;
use futures::join;
use futures::lock::MutexGuard;
use futures::StreamExt;
use node_engine::global_state::GlobalState;

use node_engine::state::{FromNodeEngine, GraphState, NodeEngineUpdate};
use sound_engine::SoundConfig;
use tokio::sync::broadcast;
use vpo_backend::io::cpal::CpalBackend;
use vpo_backend::io::file_watcher::FileWatcher;
use vpo_backend::io::load_single;
use vpo_backend::io::midir::connect_midir_backend;
use vpo_backend::util::send_graph_updates;
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
    let (midi_receiver, _midi_stream) = connect_midir_backend().unwrap();
    let (state_update_sender, state_update_receiver) = mpsc::channel();
    let (to_main, mut from_engine) = broadcast::channel(16);

    let mut backend = CpalBackend::new();
    let output_device = backend.get_default_output().unwrap();

    let (mut file_watcher, mut file_receiver) = FileWatcher::new().unwrap();

    let (_stream, config) = backend
        .connect(
            output_device,
            global_state.resources.clone(),
            engine_buffer_size,
            io_requested_buffer_size,
            48_000,
            midi_receiver,
            state_update_receiver,
            to_main,
        )
        .unwrap();

    // set up state
    global_state.sound_config = SoundConfig {
        sample_rate: config.sample_rate.0,
        buffer_size: engine_buffer_size,
    };

    let graph_state = GraphState::new(&global_state).unwrap();
    state_update_sender
        .send(vec![NodeEngineUpdate::NewNodeEngine(
            graph_state.get_engine(&global_state).unwrap(),
        )])
        .unwrap();

    println!("sample rate: {}", config.sample_rate.0);

    let global_state = futures::lock::Mutex::new(global_state);
    let graph_state = futures::lock::Mutex::new(graph_state);

    // debugging
    // let mut output_file = File::create("out.pcm").unwrap();
    join!(
        async {
            loop {
                let msg = from_server.recv().await;

                if let Ok(msg) = msg {
                    let (graph_state_lock, global_state_lock) = join!(graph_state.lock(), global_state.lock());

                    MutexGuard::map(graph_state_lock, |graph_state| {
                        MutexGuard::map(global_state_lock, |global_state| {
                            block_on(async {
                                handle_msg(
                                    msg,
                                    &to_server,
                                    graph_state,
                                    global_state,
                                    &state_update_sender,
                                    &mut file_watcher,
                                )
                                .await;
                            });

                            global_state
                        });

                        graph_state
                    });
                }
            }
        },
        async {
            while let Some(res) = file_receiver.next().await {
                match res {
                    Ok(event) => {
                        for e in event {
                            MutexGuard::map(global_state.lock().await, |global_state| {
                                let _ = load_single(&e.path, global_state);

                                global_state
                            });
                        }
                    }
                    Err(e) => println!("watch error: {:?}", e),
                }
            }
        },
        async {
            while let Ok(engine_update) = from_engine.recv().await {
                match engine_update {
                    FromNodeEngine::UiUpdates(updates) => {
                        MutexGuard::map(graph_state.lock().await, |graph_state| {
                            let root_index = graph_state.get_root_graph_index();

                            let graph = graph_state.get_graph_manager().get_graph_mut(root_index).unwrap();

                            for (node_index, new_state) in updates {
                                if let Ok(node) = graph.get_node_mut(node_index) {
                                    node.set_state(new_state);
                                }
                            }

                            send_graph_updates(graph_state, root_index, &to_server).unwrap();

                            graph_state
                        });
                    }
                    FromNodeEngine::RequestedStateUpdates(updates) => {
                        // TODO: don't unwrap here, instead recreate the engine if it fails
                        state_update_sender
                            .send(vec![NodeEngineUpdate::NewNodeState(updates)])
                            .unwrap();
                    }
                    FromNodeEngine::GraphStateRequested => {
                        MutexGuard::map(graph_state.lock().await, |graph_state| {
                            // TODO: don't unwrap here, instead recreate the engine if it fails
                            state_update_sender
                                .send(vec![NodeEngineUpdate::CurrentNodeStates(graph_state.get_node_state())])
                                .unwrap();

                            graph_state
                        });
                    }
                }
            }
        }
    );
}
