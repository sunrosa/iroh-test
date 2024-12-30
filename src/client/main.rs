use std::str::FromStr;

use iroh::{Endpoint, NodeAddr, PublicKey};

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
    let mut send = conn.open_uni().await?;

    println!("type your messages:");

    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;
        let buf = buf.trim();

        send.write_all(buf.as_bytes()).await?;
    }
}
