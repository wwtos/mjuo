use std::cell::RefCell;
use std::sync::{Arc, RwLock};

use futures::executor::LocalPool;
use futures::join;
use futures::task::LocalSpawnExt;
use futures::StreamExt;
use ipc::file_server::start_file_server;

use node_engine::resources::Resources;
use node_engine::state::{FromNodeEngine, GraphState, NodeEngineUpdate};
use sound_engine::SoundConfig;

use vpo_backend::io::cpal::CpalBackend;
use vpo_backend::io::file_watcher::FileWatcher;
use vpo_backend::io::load_single;
use vpo_backend::state::GlobalState;
use vpo_backend::util::{send_graph_updates, send_resource_updates};
use vpo_backend::{handle_msg, start_ipc};

fn main() {
    let mut async_executor = LocalPool::new();

    let (to_server, from_server, _ipc_handle) = start_ipc(26642);

    let graph_state = GraphState::new(SoundConfig::default()).unwrap();
    let global_state = GlobalState::new();
    let resources = Arc::new(RwLock::new(Resources::default()));

    // start up midi and audio
    let (midi_receiver, _midi_stream) = connect_midir_backend().unwrap();
    let (to_engine, from_main) = flume::unbounded();
    let (to_main, from_engine) = flume::unbounded();
    let (project_dir_sender, project_dir_receiver) = flume::unbounded();

    let mut backend = CpalBackend::new();
    let output_device = backend.get_default_output().unwrap();

    let (mut file_watcher, mut file_receiver) = FileWatcher::new().unwrap();

    let (_stream, config) = backend
        .connect(
            output_device,
            resources.clone(),
            graph_state.get_sound_config().buffer_size,
            1024,
            graph_state.get_sound_config().sample_rate,
            midi_receiver,
            from_main,
            to_main,
        )
        .unwrap();

    // send initial node engine instance to sound engine
    to_engine
        .send(vec![NodeEngineUpdate::NewNodeEngine(
            graph_state.get_engine(&*resources.clone().read().unwrap()).unwrap(),
        )])
        .unwrap();

    println!("sample rate: {}", config.sample_rate.0);

    let graph_state = RefCell::new(graph_state);
    let global_state = RefCell::new(global_state);

    // spawn all the async tasks
    async_executor
        .spawner()
        .spawn_local(async move {
            let client_communication = async {
                let mut project_dir_sender = project_dir_sender.clone();
                let to_server = &to_server.clone();

                loop {
                    let msg = from_server.recv_async().await.unwrap();

                    handle_msg(
                        msg,
                        &to_server,
                        &mut *graph_state.borrow_mut(),
                        &mut *global_state.borrow_mut(),
                        &*resources.clone(),
                        &to_engine,
                        &mut file_watcher,
                        &mut project_dir_sender,
                    )
                    .await;
                }
            };

            let sound_engine_communication = async {
                let to_server = to_server.clone();

                while let Ok(engine_updates) = from_engine.recv_async().await {
                    for engine_update in engine_updates {
                        match engine_update {
                            FromNodeEngine::NodeStateUpdates(updates) => {
                                let root_index = graph_state.borrow().get_root_graph_index();

                                let mut graph_state = graph_state.borrow_mut();
                                let graph = graph_state.get_graph_manager().get_graph_mut(root_index).unwrap();

                                for (node_index, new_state) in updates {
                                    if let Ok(node) = graph.get_node_mut(node_index) {
                                        node.set_state(new_state);
                                    }
                                }

                                send_graph_updates(&mut *graph_state, root_index, &to_server).unwrap();
                            }
                            FromNodeEngine::RequestedStateUpdates(updates) => {
                                // TODO: don't unwrap here, instead recreate the engine if it fails
                                to_engine.send(vec![NodeEngineUpdate::NewNodeState(updates)]).unwrap();
                            }
                            FromNodeEngine::GraphStateRequested => {
                                // TODO: don't unwrap here, instead recreate the engine if it fails
                                to_engine
                                    .send(vec![NodeEngineUpdate::CurrentNodeStates(
                                        graph_state.borrow().get_node_state(),
                                    )])
                                    .unwrap();
                            }
                        }
                    }
                }
            };

            let file_watcher = async {
                let to_server = to_server.clone();
                let resources = resources.clone();

                while let Some(res) = file_receiver.next().await {
                    match res {
                        Ok(event) => {
                            for e in event {
                                // go through all the file events and reload those resources
                                let global_state = global_state.borrow();
                                let mut resources_lock = resources.write().unwrap();

                                let root_dir = global_state.active_project.as_ref().and_then(|x| x.parent()).unwrap();
                                let _ = load_single(root_dir, &e.path, &mut *resources_lock);

                                send_resource_updates(&*resources_lock, &to_server).unwrap();
                            }
                        }
                        Err(e) => println!("watch error: {:?}", e),
                    }
                }
            };

            join!(client_communication, sound_engine_communication, file_watcher);
        })
        .unwrap();

    let _fs_handle = start_file_server(project_dir_receiver);

    async_executor.run();
}
