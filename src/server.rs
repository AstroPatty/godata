use crate::project::{get_project_manager, ProjectManager};
use crate::routes;

use std::sync::{Arc, Mutex};
use tokio_stream::wrappers::UnixListenerStream;
use tokio::signal;
use directories::UserDirs;
use sysinfo::System;


pub struct Server {
    project_manager: Arc<Mutex<ProjectManager>>,
    url: String
}




impl Server {
    pub async fn start(&self) {
        // check if the socket file already exists
        if std::path::Path::new(&self.url).exists() {
            // if it does, check if there is a "godata_server" process running
            let system = System::new();
            let mut processes = system.processes_by_name("godata_server");
            if let Some(_) = processes.next() {
                println!("A server is already running on {}", self.url);
                return;
            }
            std::fs::remove_file(&self.url).unwrap();
        }

        let listener = tokio::net::UnixListener::bind(&self.url).unwrap();
        let incoming = UnixListenerStream::new(listener);
        let server = warp::serve(routes::routes(self.project_manager.clone()))
            .serve_incoming_with_graceful_shutdown(incoming, async {
                signal::ctrl_c().await.unwrap()
            });
        server.await;
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        println!("Shutting down server...");
        std::fs::remove_file(&self.url).unwrap();
    }
}

pub fn get_server() -> Server {
    let url = UserDirs::new().unwrap().home_dir().join(".godata.sock");
    println!("Starting server on {}", url.to_str().unwrap());
    Server {
        project_manager: Arc::new(Mutex::new(get_project_manager())),
        url: url.to_str().unwrap().to_string()
    }
}
