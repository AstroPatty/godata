use crate::project::{get_project_manager, ProjectManager};
use crate::routes;

use std::sync::{Arc, Mutex};
use tokio_stream::wrappers::UnixListenerStream;
use tokio::signal;
use directories::UserDirs;
use sysinfo::System;


pub struct Server {
    project_manager: Arc<Mutex<ProjectManager>>,
    url: (String, Option<u16>)
}




impl Server {
    pub async fn start(&self) {
        // If there's a port, start a TCP server
        if self.url.1.is_some() {
            println!("Starting server on {}", self.url.1.unwrap());
            let (_, server) = warp::serve(routes::routes(self.project_manager.clone()))
                .bind_with_graceful_shutdown(([127, 0, 0, 1], self.url.1.unwrap()), async {
                    signal::ctrl_c().await.unwrap()
                });
            server.await
        }

        // If there's no port, start a Unix socket server

        else {
            if std::path::Path::new(&self.url.0).exists() {
                // check if the socket file already exists
                // if it does, check if there is a "godata_server" process running
                let system = System::new();
                let mut processes = system.processes_by_name("godata_server");
                if let Some(_) = processes.next() {
                    println!("A server is already running on {}", self.url.0);
                    return;
                }
                std::fs::remove_file(&self.url.0).unwrap();
            }
            let listener = tokio::net::UnixListener::bind(&self.url.0).unwrap();
            let incoming = UnixListenerStream::new(listener);
            let server = warp::serve(routes::routes(self.project_manager.clone()))
                .serve_incoming_with_graceful_shutdown(incoming, async {
                    signal::ctrl_c().await.unwrap()
                });
            server.await
        };
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        println!("Shutting down server...");
        if self.url.1.is_some() {
            return;
        }
        std::fs::remove_file(&self.url.0).unwrap();
    }
}

pub fn get_server(port: Option<u16>) -> Server {
    let url = match port {
        Some(p) => format!("localhost:{}", p),
        None => UserDirs::new().unwrap().home_dir().join(".godata.sock").to_str().unwrap().to_string()
    };
    println!("Starting godata server on {}", url);
    Server {
        project_manager: Arc::new(Mutex::new(get_project_manager())),
        url: (url, port),
    }
}
