use crate::project::{get_project_manager, ProjectManager};
use crate::routes;

use std::sync::{Arc, Mutex};
use tokio_stream::wrappers::UnixListenerStream;
use tokio::signal;
use directories::UserDirs;


pub struct Server {
    project_manager: Arc<Mutex<ProjectManager>>,
    url: String
}




impl Server {
    pub async fn start(&self) {
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
    Server {
        project_manager: Arc::new(Mutex::new(get_project_manager())),
        url: url.to_str().unwrap().to_string()
    }
}
