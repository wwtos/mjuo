use std::sync::{mpsc, Arc, RwLock};

use futures::executor::block_on;
use futures::join;
use futures::lock::MutexGuard;
use futures::StreamExt;
use tokio::sync::broadcast;

use node_engine::global_state::{GlobalState, Resources};
use node_engine::state::{FromNodeEngine, GraphState, NodeEngineUpdate};
use sound_engine::SoundConfig;

use tower_http::services::ServeDir;
use vpo_backend::io::cpal::CpalBackend;
use vpo_backend::io::file_watcher::FileWatcher;
use vpo_backend::io::load_single;
use vpo_backend::io::midir::connect_midir_backend;
use vpo_backend::util::{send_global_state_updates, send_graph_updates};
use vpo_backend::{handle_msg, start_ipc};

#[tokio::main]
async fn main() {
    main_async().await;
}

async fn main_async() {
    let (to_server, mut from_server) = start_ipc(26642);

    let engine_buffer_size = 64;
    let io_requested_buffer_size = 1024;

    let mut global_state = GlobalState::new(SoundConfig::default());
    let resources = Arc::new(RwLock::new(Resources::default()));

    // start up midi and audio
    let (midi_receiver, _midi_stream) = connect_midir_backend().unwrap();
    let (to_engine, from_main) = mpsc::channel();
    let (to_main, mut from_engine) = broadcast::channel(128);
    let (project_dir_sender, mut project_dir_receiver) = broadcast::channel(16);

    let mut backend = CpalBackend::new();
    let output_device = backend.get_default_output().unwrap();

    let (mut file_watcher, mut file_receiver) = FileWatcher::new().unwrap();

    let (_stream, config) = backend
        .connect(
            output_device,
            resources.clone(),
            engine_buffer_size,
            io_requested_buffer_size,
            48_000,
            midi_receiver,
            from_main,
            to_main,
        )
        .unwrap();

    // set up state
    global_state.sound_config = SoundConfig {
        sample_rate: config.sample_rate.0,
        buffer_size: engine_buffer_size,
    };

    let graph_state = GraphState::new(&global_state).unwrap();
    to_engine
        .send(vec![NodeEngineUpdate::NewNodeEngine(
            graph_state
                .get_engine(&global_state, &*resources.read().unwrap())
                .unwrap(),
        )])
        .unwrap();

    println!("sample rate: {}", config.sample_rate.0);

    let global_state = futures::lock::Mutex::new(global_state);
    let graph_state = futures::lock::Mutex::new(graph_state);

    let message_receiving_block = async {
        let mut project_dir_sender = project_dir_sender.clone();

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
                                &*resources.clone(),
                                &to_engine,
                                &mut file_watcher,
                                &mut project_dir_sender,
                            )
                            .await;
                        });

                        global_state
                    });

                    graph_state
                });
            }
        }
    };

    let engine_message_receiving_block = async {
        while let Ok(engine_updates) = from_engine.recv().await {
            MutexGuard::map(graph_state.lock().await, |graph_state| {
                for engine_update in engine_updates {
                    match engine_update {
                        FromNodeEngine::NodeStateUpdates(updates) => {
                            let root_index = graph_state.get_root_graph_index();

                            let graph = graph_state.get_graph_manager().get_graph_mut(root_index).unwrap();

                            for (node_index, new_state) in updates {
                                if let Ok(node) = graph.get_node_mut(node_index) {
                                    node.set_state(new_state);
                                }
                            }

                            send_graph_updates(graph_state, root_index, &to_server).unwrap();
                        }
                        FromNodeEngine::RequestedStateUpdates(updates) => {
                            // TODO: don't unwrap here, instead recreate the engine if it fails
                            to_engine.send(vec![NodeEngineUpdate::NewNodeState(updates)]).unwrap();
                        }
                        FromNodeEngine::GraphStateRequested => {
                            // TODO: don't unwrap here, instead recreate the engine if it fails
                            to_engine
                                .send(vec![NodeEngineUpdate::CurrentNodeStates(graph_state.get_node_state())])
                                .unwrap();
                        }
                    }
                }
                graph_state
            });
        }
    };

    let file_watcher_block = async {
        let to_server = to_server.clone();

        while let Some(res) = file_receiver.next().await {
            match res {
                Ok(event) => {
                    for e in event {
                        MutexGuard::map(global_state.lock().await, |global_state| {
                            let root = global_state.active_project.as_ref().and_then(|x| x.parent()).unwrap();
                            let _ = load_single(root, &e.path, &mut *resources.clone().write().unwrap());

                            send_global_state_updates(global_state, &to_server).unwrap();

                            global_state
                        });
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    };

    let fs_block = async {
        let mut updated_project_receiver = project_dir_sender.subscribe();

        loop {
            updated_project_receiver.recv().await.expect("not closed");
            let project_dir = project_dir_receiver.recv().await.expect("not closed");

            let service = ServeDir::new(project_dir);

            let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 26643));
            let server = async {
                hyper::Server::bind(&addr)
                    .serve(tower::make::Shared::new(service))
                    .await
                    .expect("server error")
            };

            tokio::select! {
                _ = updated_project_receiver.recv() => {
                    // if a new project dir comes in, it'll drop the file server
                }
                _ = server => {}
            }
        }
    };

    // debugging
    // let mut output_file = File::create("out.pcm").unwrap();
    join!(
        message_receiving_block,
        file_watcher_block,
        engine_message_receiving_block,
        fs_block
    );
}
