
mod project;
mod fsystem;
mod storage;
mod locations;
mod server;
mod routes;
mod handlers;

#[tokio::main]
async fn main () {
    let srv = server::get_server();
    srv.start().await;
}


