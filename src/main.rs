use crate::client::ClientOpts;
use crate::server::ServerOpts;
use clap::Parser;

mod client;
mod server;

#[derive(Parser, Debug)]
#[command(name = "netgap", version, author, about, long_about = None)]
struct Command {
    #[command(subcommand)]
    cmd: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
    #[command(about = "Run as a server")]
    Server(ServerOpts),
    #[command(about = "Run as a client")]
    Client(ClientOpts),
}

#[derive(Debug)]
struct Metrics {
    total: usize,
    success: usize,
    failed: usize,
    bytes: usize,
}

fn main() {
    let command = Command::parse();

    match command.cmd {
        SubCommand::Server(opts) => {
            server::run(opts).expect("server error");
        }
        SubCommand::Client(opts) => {
            client::run(opts).expect("client error");
        }
    }
}
