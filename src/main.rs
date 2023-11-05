use std::collections::HashMap;

use anyhow::{Context, Result};
use dns::{message::Message, DeSerialize, Serialize};
use tokio::net::UdpSocket;

use crate::dns::{header::Header, question::Question, record::Record, Global};

mod dns;

#[tokio::main]
async fn main() -> Result<()> {
    let sock = UdpSocket::bind("0.0.0.0:8080")
        .await
        .context("could not bind")?;

    let m = Message::single("tandolf.github.io");
    let m = m.serialize().unwrap();

    let mut buffer = [0; 128];
    let _len = sock.send_to(&m, "1.1.1.1:53").await?;
    let (_, _) = sock.recv_from(&mut buffer).await?;

    // for b in &buf[..len] {
    //     print!("0x{:02x}, ", *b);
    // }

    let mut global = Global {
        cache: HashMap::new(),
        source: &buffer,
    };
    let (buffer, header) = Header::deserialize(&buffer, &mut global).unwrap();
    let (buffer, question) = Question::deserialize(buffer, &mut global).unwrap();
    let (_buffer, record) = Record::deserialize(buffer, &mut global).unwrap();

    dbg!(&header);
    dbg!(&question);
    dbg!(&record);

    // let response = DNSResponse::deserialize(&buf);

    // println!("{:?}", response);
    //

    Ok(())
}
