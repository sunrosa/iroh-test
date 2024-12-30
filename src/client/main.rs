use std::str::{from_utf8, FromStr};

use iroh::{
    endpoint::{Connection, RecvStream},
    Endpoint, NodeAddr, PublicKey,
};

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

    tokio::task::spawn(receiver(conn.clone()));

    println!("type your messages:");

    let mut send = conn.open_uni().await?;
    send.write_all("connected".as_bytes()).await?;
    send.finish()?;

    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;
        let buf = buf.trim();

        let mut send = conn.open_uni().await?;
        send.write_all(buf.as_bytes()).await?;
        send.finish()?;
    }
}

async fn receiver(conn: Connection) -> anyhow::Result<()> {
    loop {
        let mut recv = conn.accept_uni().await?;

        let received = recv.read_to_end(1024).await?;
        let utf8 = from_utf8(&received)?.trim();

        if !utf8.is_empty() {
            // If a message was received and has a length greater than 0, print it out
            // Checking against buf[0] being 0 because recv.read will read twice per actual received message for some reason. The second message is zeroed.
            println!("{utf8}");
        }
    }
}
