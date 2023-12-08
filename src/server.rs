use crate::project::{get_project_manager, ProjectManager};
use crate::routes;

use std::sync::{Arc, Mutex};
use tokio_stream::wrappers::UnixListenerStream;
use tokio::signal;



pub struct Server {
    project_manager: Arc<Mutex<ProjectManager>>
}




impl Server {
    pub async fn start(&self) {
        let listener = tokio::net::UnixListener::bind("/tmp/godata.sock").unwrap();
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
        std::fs::remove_file("/tmp/godata.sock").unwrap();
    }
}

pub fn get_server() -> Server {
    let server = Server {
        project_manager: Arc::new(Mutex::new(get_project_manager()))
    };
    server
}
