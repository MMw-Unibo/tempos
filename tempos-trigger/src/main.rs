use clap::Parser;
use std::{
    io::Write,
    net::{SocketAddr, UdpSocket},
};

use tempos::msg_type;

/// Simple TEMPOS Trigger example
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Topic to send the message
    #[clap(short, long)]
    topic: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let addr: SocketAddr = "127.0.0.1:50000".parse()?;
    let sock = UdpSocket::bind(&addr)?;

    let mut buf: Vec<u8> = Vec::with_capacity(1024);

    let topic = args.topic.clone();
    let topic_len = args.topic.len() as u32;

    buf.write(&msg_type::INVOK.to_be_bytes())?;
    buf.write(&topic_len.to_be_bytes())?;
    buf.write(&topic.as_bytes())?;

    sock.send_to(&buf, "127.0.0.1:8080")?;

    Ok(())
}
