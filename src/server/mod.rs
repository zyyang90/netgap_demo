use clap::Parser;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

#[derive(Parser, Debug, Clone)]
pub struct ServerOpts {
    /// The port to listen
    #[arg(short, long, default_value_t = 8899)]
    port: u16,
    /// Whether to send ack to the client
    #[arg(short, long, default_value_t = true)]
    ack: bool,
}

pub fn run(opts: ServerOpts) -> anyhow::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", opts.port))
        .map_err(|err| anyhow::anyhow!("failed to bind to address: {}", err))?;
    println!("Server started at 127.0.0.1:{}", opts.port);

    for stream in listener.incoming() {
        let stream = stream.map_err(|err| anyhow::anyhow!("failed to get stream: {}", err))?;
        println!(
            "New connection: {}",
            stream
                .peer_addr()
                .map_err(|err| { anyhow::anyhow!("failed to get peer address: {}", err) })?
        );

        let opts = opts.clone();
        thread::spawn(move || {
            handle_connection(opts, stream).unwrap();
        });
    }

    drop(listener);

    Ok(())
}

fn handle_connection(opts: ServerOpts, mut stream: TcpStream) -> anyhow::Result<()> {
    let mut buffer = [0u8; 996];

    const OK: &[u8] = &[0b1111_1111u8];
    const FAIL: &[u8] = &[0b0000_0000u8];

    loop {
        match stream.read(&mut buffer) {
            Ok(size) => {
                if size == 0 {
                    println!("connection closed");
                    stream.shutdown(Shutdown::Both).map_err(|err| {
                        anyhow::anyhow!("failed to shutdown the stream, cause: {:?}", err)
                    })?;
                }

                let content = String::from_utf8_lossy(&buffer[..size]);
                println!("received size: {}, content: {}", size, content);
                if opts.ack {
                    stream.write(OK).map_err(|err| {
                        anyhow::anyhow!("failed to write data to the client, cause: {:?}", err)
                    })?;
                }
            }
            Err(err) => {
                println!("failed to read data, cause: {:#?}", err);
                stream.write(FAIL).map_err(|err| {
                    anyhow::anyhow!("failed to write data to the client, cause: {:?}", err)
                })?;
                stream.shutdown(Shutdown::Both).map_err(|err| {
                    anyhow::anyhow!("failed to shutdown the stream, cause: {:?}", err)
                })?;
            }
        }
    }
}
