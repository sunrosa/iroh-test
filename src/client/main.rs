use std::str::FromStr;

use iroh_net::{key::PublicKey, Endpoint, NodeAddr};

fn main() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    println!("enter address:");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    let buf = buf.trim();

    // let buf_decoded = base32::decode(Rfc4648Lower { padding: false }, &buf)
    //     .ok_or(anyhow!("could not decode base32"))?;

    let addr = NodeAddr::new(PublicKey::from_str(buf)?);
    let ep = Endpoint::builder().discovery_n0().bind().await?;
    let conn = ep.connect(addr, b"my-alpn").await?;
    let mut send = conn.open_uni().await?;

    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;
        let buf = buf.trim();

        send.write_all(buf.as_bytes()).await?;
    }
}
