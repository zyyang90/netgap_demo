use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use clap::Parser;

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

#[derive(Parser, Debug)]
struct ServerOpts {
    #[arg(short, long, default_value = "8899")]
    port: u16,
}

#[derive(Parser, Debug)]
struct ClientOpts {
    #[arg(long)]
    host: String,
    #[arg(short, long, default_value = "8899")]
    port: u16,
}

fn main() {
    let command = Command::parse();

    match command.cmd {
        SubCommand::Server(opts) => {
            server(opts).expect("server error");
        }
        SubCommand::Client(opts) => {
            client(opts).expect("client error");
        }
    }
}

fn client(opts: ClientOpts) -> anyhow::Result<()> {
    match TcpStream::connect(format!("{}:{}", opts.host, opts.port)) {
        Ok(mut stream) => {
            println!("connected to the server: {}:{}", opts.host, opts.port);

            let msg = "Hello, server!";

            for _ in 0..10 {
                stream.write(msg.as_bytes()).unwrap();
                stream.flush().unwrap();
                println!(">>> {}", msg);

                thread::sleep(Duration::from_secs(1));
            }
        }
        Err(e) => {
            println!("failed to connect to the server, cause: {:#?}", e);
        }
    }
    Ok(())
}
fn server(opts: ServerOpts) -> anyhow::Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", opts.port))
        .map_err(|err| anyhow::anyhow!("failed to bind to address: {}", err))?;
    println!("Server started at 127.0.0.1:{}", opts.port);

    for stream in listener.incoming() {
        let stream = stream.map_err(|err| anyhow::anyhow!("failed to get stream: {}", err))?;
        println!("New connection: {}", stream.peer_addr().unwrap());

        thread::spawn(move || {
            handle_connection(stream);
        });
    }

    drop(listener);

    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0u8; 1024];

    const OK: &[u8] = &[0b1111_1111u8];
    const FAIL: &[u8] = &[0b0000_0000u8];

    loop {
        match stream.read(&mut buffer) {
            Ok(size) => {
                if size == 0 {
                    println!("connection closed");
                    std::process::exit(0);
                }

                let content = String::from_utf8_lossy(&buffer[..size]);
                println!("received size: {}, content: {}", size, content);
                // stream.write(&[OK]).unwrap();
            }
            Err(err) => {
                println!("failed to read data, cause: {:#?}", err);
                // stream.write(&[FAIL]).unwrap();
                stream.shutdown(Shutdown::Both).unwrap();
            }
        }
    }
}
