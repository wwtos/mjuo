use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;

use env_logger::Env;
use futures::executor::LocalPool;
use futures::join;
use futures::task::LocalSpawnExt;
use futures::StreamExt;
use ipc::file_server::{start_file_server, start_file_server_in};

use node_engine::resources::Resources;
use node_engine::state::{FromNodeEngine, GraphState};
use sound_engine::SoundConfig;

use thread_priority::{ThreadBuilderExt, ThreadPriority};
use vpo_backend::engine::{start_sound_engine, ToAudioThread};
use vpo_backend::io::file_watcher::FileWatcher;
use vpo_backend::io::load_single;
use vpo_backend::state::GlobalState;
use vpo_backend::util::{send_graph_updates, send_resource_updates};
use vpo_backend::{handle_msg, start_ipc};

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();

    let (to_server, from_server, _ipc_handle) = start_ipc(26642);

    let global_state = RefCell::new(GlobalState::new());
    let graph_state = RefCell::new(GraphState::new(SoundConfig::default()));
    let resources = Arc::new(RwLock::new(Resources::default()));

    let (to_realtime, from_main) = flume::unbounded();
    let (to_main, from_realtime) = flume::unbounded();
    let (project_dir_sender, project_dir_receiver) = flume::unbounded();

    let (mut file_watcher, mut from_file_watcher) = FileWatcher::new().unwrap();
    let _project_files_handle = start_file_server(project_dir_receiver, 26643);
    let _frontend_server_handle = start_file_server_in(PathBuf::from("./frontend"), 26644);

    println!("Welcome to MJUO! Please go to http://localhost:26644 in your browser to see the UI");

    let resources_for_audio_thread = resources.clone();

    thread::Builder::new()
        .name("audio_thread".into())
        .spawn_with_priority(ThreadPriority::Max, move |_| {
            start_sound_engine(resources_for_audio_thread, from_main, to_main);
        })
        .unwrap();

    // spawn all the async tasks
    let mut async_executor = LocalPool::new();
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
                        &to_realtime,
                        &mut file_watcher,
                        &mut project_dir_sender,
                    )
                    .await;
                }
            };

            let sound_engine_communication = async {
                let to_server = to_server.clone();

                while let Ok(engine_update) = from_realtime.recv_async().await {
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
                            let mut graph_state = graph_state.borrow_mut();
                            let root_index = graph_state.get_root_graph_index();

                            let root = graph_state.get_graph_manager().get_graph_mut(root_index).unwrap();

                            for (node_index, value) in &updates {
                                if let Ok(node) = root.get_node_mut(*node_index) {
                                    node.get_state_mut().value = value.clone();
                                }
                            }

                            to_realtime.send(ToAudioThread::NewNodeStates(updates)).unwrap();

                            send_graph_updates(&mut *graph_state, root_index, &to_server).unwrap();
                        }
                        FromNodeEngine::GraphStateRequested => {
                            // TODO: don't unwrap here, instead recreate the engine if it fails
                            to_realtime
                                .send(ToAudioThread::CurrentNodeStates(graph_state.borrow().get_node_state()))
                                .unwrap();
                        }
                    }
                }
            };

            let file_watcher = async {
                let to_server = to_server.clone();
                let resources = resources.clone();

                while let Some(res) = from_file_watcher.next().await {
                    match res {
                        Ok(event) => {
                            for e in event {
                                // go through all the file events and reload those resources
                                let global_state = global_state.borrow();
                                let graph_state = graph_state.borrow();
                                let mut resources_lock = resources.write().unwrap();

                                let root_dir = global_state.active_project.as_ref().and_then(|x| x.parent()).unwrap();
                                let _ = load_single(
                                    root_dir,
                                    &e.path,
                                    &mut *resources_lock,
                                    graph_state.get_sound_config(),
                                );

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

    async_executor.run();
}
