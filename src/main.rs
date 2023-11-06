use anyhow::{Context, Result};
use dns::{message::Message, DeSerialize, Serialize};
use tokio::net::UdpSocket;

use crate::dns::Buffer;

mod dns;

#[tokio::main]
async fn main() -> Result<()> {
    let sock = UdpSocket::bind("0.0.0.0:8080")
        .await
        .context("could not bind")?;

    let m = Message::single("blog.toerktumlare.com");
    let m = m.serialize().unwrap();

    let mut buffer = [0; 1024];
    let _len = sock.send_to(&m, "1.1.1.1:53").await?;
    let (_, _) = sock.recv_from(&mut buffer).await?;

    let mut buffer = Buffer {
        current: &buffer,
        source: &buffer,
    };

    let (_buffer, message) = Message::deserialize(&mut buffer).unwrap();

    dbg!(&message);
    Ok(())
}
