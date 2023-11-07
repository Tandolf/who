use std::process;

use anyhow::{Context, Result};
use clap::Parser;
use dns::{message::Message, DeSerialize, Serialize};
use tokio::net::UdpSocket;

use crate::dns::Buffer;

mod dns;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    address: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.address.is_empty() {
        eprintln!("please provide a valid address");
        process::exit(1);
    }

    let m = Message::single(args.address);

    let sock = UdpSocket::bind("0.0.0.0:8080")
        .await
        .context("could not bind")?;

    // let m = Message::txt("toerktumlare.com");
    let m = m.serialize().unwrap();

    let mut buffer = [0; 1024];
    let _len = sock.send_to(&m, "1.1.1.1:53").await?;
    let (_, _) = sock.recv_from(&mut buffer).await?;

    let mut buffer = Buffer {
        current: &buffer,
        source: &buffer,
    };

    let (_buffer, message) = Message::deserialize(&mut buffer).unwrap();

    println!("{}", message);
    Ok(())
}
