use clap::Parser;
use rand::prelude::*;
use std::{
    io::Write,
    net::{self, SocketAddr},
    sync::{atomic::AtomicBool, Arc},
    thread,
    time::Duration,
};

/// Simple TEMPOS Invoker example
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Topic to send the message
    #[clap(short, long)]
    node: u32,
    #[clap(short, long)]
    topic: String,
}

pub fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;

    let sock = net::UdpSocket::bind("127.0.0.1:9001")?;
    sock.set_read_timeout(Some(Duration::from_millis(100)))?;

    let mut buf: Vec<u8> = Vec::with_capacity(1024);

    buf.write(&tempos::msg_type::REGISTRATION.to_be_bytes())?;
    buf.write(&args.node.to_be_bytes())?;

    let topic = args.topic.clone();
    let topic_len = args.topic.len() as u32;
    buf.write(&topic_len.to_be_bytes())?;
    buf.write(&topic.as_bytes())?;

    sock.send_to(&buf, addr)?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::Relaxed);
    })?;

    let r = running.clone();

    let socket = sock.try_clone()?;

    thread::spawn(move || {
        let mut buf: Vec<u8> = Vec::with_capacity(1024);
        while r.load(std::sync::atomic::Ordering::Relaxed) {
            log::debug!("waiting for message");
            match socket.recv_from(&mut buf) {
                Ok((size, _)) => {
                    log::info!("received {} bytes: {:?}", size, buf);
                    let msg_type = u8::from_be_bytes([buf[0]]);
                    match msg_type {
                        tempos::msg_type::INVOK => {
                            let topic_len = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
                            let topic =
                                std::str::from_utf8(&buf[5..5 + topic_len as usize]).unwrap();
                            log::info!("invoking topic: {}", topic);
                        }
                        _ => println!("Unhandled message type"),
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // timeout
                }
                Err(e) => {
                    println!("encountered IO error: {}", e);
                }
            }
        }
    });

    let r = running.clone();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let sock = tokio::net::UdpSocket::bind("127.0.0.1:9002".parse::<SocketAddr>().unwrap())
                .await
                .unwrap();

            let mut buf = Vec::with_capacity(1024);

            while r.load(std::sync::atomic::Ordering::Relaxed) {
                buf.write(&tempos::msg_type::MONITORING.to_be_bytes())
                    .unwrap();
                buf.write(&args.node.to_be_bytes()).unwrap();
                let load: f32 = random();
                buf.write(&load.to_be_bytes()).unwrap();

                println!("sending = {:?}", buf);

                sock.send_to(&mut buf, addr).await.unwrap();

                tokio::time::sleep(Duration::from_secs(1)).await;

                buf.clear();
            }
        });

    buf.clear();
    buf.write(&tempos::msg_type::UNREGISTRATION.to_be_bytes())
        .unwrap();
    buf.write(&args.node.to_be_bytes()).unwrap();

    sock.send_to(&buf, addr).unwrap();

    log::info!("exiting...");

    Ok(())
}
