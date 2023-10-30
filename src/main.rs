use anyhow::{Context, Result};
use dns::{message::DNSPackage, DeSerialize, Serialize};
use rand::Rng;
use tokio::net::UdpSocket;

mod dns;

#[tokio::main]
async fn main() -> Result<()> {
    let mut rng = rand::thread_rng();

    let id: u16 = rng.gen_range(1..65535);
    let id: &[u8] = &id.to_be_bytes();
    let h = &[0x01_u8, 0x20_u8];
    let qd_count = 0x01;
    let padding = 0x00;
    let google = b"google";
    let google_len = google.len().try_into().context("len to u8")?;

    let com = b"com";
    let com_len = com.len().try_into().context("len to u8")?;

    let header = &[
        id[0], id[1], h[0], h[1], padding, qd_count, padding, padding, padding, padding, padding,
        qd_count,
    ];

    let mut n = google.to_vec();
    n.insert(0, google_len);
    n.append(&mut com.to_vec());
    n.insert(7, com_len);

    n.push(0x00);
    n.push(0x00);
    n.push(0x01);
    n.push(0x00);
    n.push(0x01);

    let m: &[u8] = &n;

    let _msg = [header, m].concat();

    let sock = UdpSocket::bind("0.0.0.0:8080")
        .await
        .context("could not bind")?;

    let m = DNSPackage::single("google.com");
    let m = m.serialize().unwrap();

    let mut buf = [0; 4096];
    let _len = sock.send_to(&m, "1.1.1.1:53").await?;
    let (len, _) = sock.recv_from(&mut buf).await?;

    let response = DNSPackage::deserialize(&buf);

    for b in &buf[..len] {
        print!("0x{:02x}, ", *b);
    }

    println!("{:?}", response);

    Ok(())
}
