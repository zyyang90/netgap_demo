use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8899")
        .map_err(|err| anyhow::anyhow!("failed to bind to address: {}", err))?;

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

    const OK: u8 = 0b1111_1111u8;
    const FAIL: u8 = 0b0000_0000u8;

    while match stream.read(&mut buffer) {
        Ok(size) => {
            let content = String::from_utf8_lossy(&buffer[..size]);
            println!("received size: {},content: {}", size, content);
            stream.write(&[OK]).unwrap();
            true
        }
        Err(err) => {
            println!("failed to read data, cause: {:#?}", err);
            stream.write(&[FAIL]).unwrap();
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}
