mod fsystem;
mod handlers;
mod locations;
mod project;
mod routes;
mod server;
mod storage;
mod log;

use clap::Parser;
// Allow the server to return its version with a --version flag
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
struct Opts {
    #[clap(short, long)]
    version: bool,
    #[clap(short, long)]
    debug: bool,
    #[clap(short, long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();
    if opts.version {
        println!("{}", VERSION);
        return;
    }
    let _log_guard = log::init_logging();
    let srv = server::get_server(opts.port);
    srv.start().await;
}
