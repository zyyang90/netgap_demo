use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use crate::Metrics;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct ClientOpts {
    /// The host to connect
    #[arg(long)]
    host: String,
    /// The port to connect
    #[arg(short, long, default_value_t = 8899)]
    port: u16,
    /// The count of channels to send
    #[arg(short, long, default_value_t = 1)]
    channels: usize,
    /// The total count of messages to send
    #[arg(long, default_value_t = 10)]
    msg_total: usize,
    /// The length of each message
    #[arg(long, default_value_t = 996)]
    msg_length: usize,
    /// The interval between each message
    #[arg(long, default_value_t = 1000)]
    msg_interval_ms: usize,
    /// The read timeout in milliseconds
    #[arg(long, default_value_t = 1000)]
    read_timeout: u64,
}

pub fn run(opts: ClientOpts) -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    let threads: Vec<_> = (0..opts.channels)
        .map(|_| {
            let opts = opts.clone();
            thread::spawn(move || run_impl(opts).unwrap())
        })
        .collect();

    let mut metrics = Metrics {
        total: 0,
        success: 0,
        failed: 0,
        bytes: 0,
    };

    for t in threads {
        match t.join() {
            Ok(m) => {
                metrics.total += m.total;
                metrics.success += m.success;
                metrics.failed += m.failed;
                metrics.bytes += m.bytes;
            }
            Err(e) => {
                println!("A thread panicked: {:?}", e);
            }
        }
    }

    let elapsed = start.elapsed();
    let rate = metrics.bytes as f64 / elapsed.as_secs_f64();
    let (rate, unit) = if rate < 1024.0 {
        (rate, "B/s")
    } else if rate < 1024.0 * 1024.0 {
        (rate / 1024.0, "KB/s")
    } else if rate < 1024.0 * 1024.0 * 1024.0 {
        (rate / (1024.0 * 1024.0), "MB/s")
    } else {
        (rate / (1024.0 * 1024.0 * 1024.0), "GB/s")
    };

    println!("=====================");
    println!("{:?}", metrics);
    println!("time cost: {:?}, rate: {:.2} {}", elapsed, rate, unit);
    println!("=====================");

    Ok(())
}

fn run_impl(opts: ClientOpts) -> anyhow::Result<Metrics> {
    let mut metrics = Metrics {
        total: 0,
        success: 0,
        failed: 0,
        bytes: 0,
    };

    match TcpStream::connect(format!("{}:{}", opts.host, opts.port)) {
        Ok(mut stream) => {
            println!("connected to the server: {}:{}", opts.host, opts.port);
            stream
                .set_read_timeout(Some(Duration::from_millis(opts.read_timeout)))
                .map_err(|err| anyhow::anyhow!("failed to set read timeout, cause: {}", err))?;

            let msg = vec![0u8; opts.msg_length];
            'SEND: loop {
                if metrics.total >= (opts.msg_total / opts.channels) {
                    break 'SEND;
                }
                metrics.total += 1;

                // write to server
                stream.write(msg.as_ref()).map_err(|err| {
                    anyhow::anyhow!("failed to write data to the server, cause: {}", err)
                })?;
                stream.flush().map_err(|err| {
                    anyhow::anyhow!("failed to flush data to the server, cause: {}", err)
                })?;
                metrics.bytes += opts.msg_length;

                // check ack
                let mut buffer = [0u8; 1];
                match stream.read_exact(&mut buffer) {
                    Ok(_) => {
                        if buffer[0] == 0b1111_1111 {
                            metrics.success += 1;
                        } else if buffer[0] == 0b0000_0000 {
                            metrics.failed += 1;
                        } else {
                            println!("unknown ack: {:?}", buffer);
                        }
                    }
                    Err(err) => {
                        println!("failed to read data from the server, cause: {:#?}", err);
                    }
                }

                if opts.msg_interval_ms > 0 {
                    thread::sleep(Duration::from_millis(opts.msg_interval_ms as u64));
                }
            }
        }
        Err(e) => {
            println!("failed to connect to the server, cause: {:#?}", e);
        }
    }

    Ok(metrics)
}
