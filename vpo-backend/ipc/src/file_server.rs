use std::{
    path::PathBuf,
    thread::{self, JoinHandle},
};

use tokio::runtime;
use tower_http::services::ServeDir;

pub fn start_file_server(root_folder: flume::Receiver<PathBuf>) -> JoinHandle<()> {
    thread::spawn(move || {
        let rt = runtime::Builder::new_current_thread().enable_io().build().unwrap();

        let updated_project_receiver = root_folder.clone();

        rt.block_on(async {
            loop {
                updated_project_receiver.recv_async().await.expect("not closed");
                let project_dir = root_folder.recv_async().await.expect("not closed");

                let service = ServeDir::new(project_dir);

                let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 26643));
                let server = async {
                    hyper::Server::bind(&addr)
                        .serve(tower::make::Shared::new(service))
                        .await
                        .expect("server error")
                };

                tokio::select! {
                    _ = updated_project_receiver.recv_async() => {
                        // if a new project dir comes in, it'll drop the file server
                    }
                    _ = server => {}
                }
            }
        });
    })
}
