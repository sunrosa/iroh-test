use std::str::{from_utf8, FromStr};

use iroh::{endpoint::RecvStream, Endpoint, NodeAddr, PublicKey};

fn main() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    let public_key = include_str!("public_key").trim();

    let addr = NodeAddr::new(PublicKey::from_str(public_key)?);
    let ep = Endpoint::builder().discovery_n0().bind().await?;
    let conn = ep.connect(addr, b"my-alpn").await?;
    let (mut send, recv) = conn.open_bi().await?;

    tokio::task::spawn(receiver(recv));

    println!("type your messages:");

    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;
        let buf = buf.trim();

        send.write_all(buf.as_bytes()).await?;
    }
}

async fn receiver(mut recv: RecvStream) -> anyhow::Result<()> {
    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        recv.read(&mut buf).await?;

        let utf8 = from_utf8(&buf)?.trim();

        println!("{utf8}");
    }
}
