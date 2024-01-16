mod project;
mod fsystem;
mod storage;
mod locations;
mod server;
mod routes;
mod handlers;

use clap::Parser;
// Allow the server to return its version with a --version flag
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
struct Opts {
    #[clap(short, long)]
    version: bool
}


#[tokio::main]
async fn main () {
    let opts: Opts = Opts::parse();
    if opts.version {
        println!("{}", VERSION);
        return;
    }
    let srv = server::get_server();
    srv.start().await;
}


