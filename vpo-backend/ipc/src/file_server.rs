use std::{
    path::PathBuf,
    thread::{self, JoinHandle},
};

use tokio::runtime;
use tower_http::services::ServeDir;

pub fn start_file_server_in(root_folder: PathBuf, port: u16) -> JoinHandle<()> {
    let (sender, receiver) = flume::unbounded();
    sender.send(root_folder).unwrap();

    start_file_server(receiver, port)
}

pub fn start_file_server(root_folder: flume::Receiver<PathBuf>, port: u16) -> JoinHandle<()> {
    thread::spawn(move || {
        let rt = runtime::Builder::new_current_thread().enable_io().build().unwrap();

        rt.block_on(async {
            // wait for the first folder
            let mut project_dir = root_folder.recv_async().await.expect("not closed");

            loop {
                let service = ServeDir::new(project_dir.clone());

                let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
                let server = async {
                    hyper::Server::bind(&addr)
                        .serve(tower::make::Shared::new(service))
                        .await
                        .expect("server error")
                };

                // use select to terminate the server early
                tokio::select! {
                    Ok(new_project_dir) = root_folder.recv_async() => {
                        project_dir = new_project_dir;
                        // if a new project dir comes in, it'll drop the file server
                    }
                    _ = server => {
                        unreachable!("server should never stop");
                    }
                }
            }
        });
    })
}
