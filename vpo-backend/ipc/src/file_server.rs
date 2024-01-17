use std::{
    path::PathBuf,
    thread::{self, JoinHandle},
};

use tokio::runtime;
use tower_http::services::ServeDir;

pub fn start_file_server(root_folder: flume::Receiver<PathBuf>) -> JoinHandle<()> {
    thread::spawn(move || {
        let rt = runtime::Builder::new_current_thread().enable_io().build().unwrap();

        rt.block_on(async {
            let mut project_dir = root_folder.recv_async().await.expect("not closed");

            loop {
                let service = ServeDir::new(project_dir.clone());

                let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 26643));
                let server = async {
                    hyper::Server::bind(&addr)
                        .serve(tower::make::Shared::new(service))
                        .await
                        .expect("server error")
                };

                // use select to terminate the server early
                tokio::select! {
                    new_project_dir = root_folder.recv_async() => {
                        project_dir = new_project_dir.expect("not closed");
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
