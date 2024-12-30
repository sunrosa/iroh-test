use std::str::from_utf8;

use anyhow::anyhow;
use iroh_net::Endpoint;

fn main() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    let ep = Endpoint::builder()
        .alpns(vec![b"my-alpn".to_vec()])
        .discovery_n0()
        .bind()
        .await?;

    println!("node id: {}\nawaiting connection...", ep.node_id());

    let conn = ep.accept().await.ok_or(anyhow!("err"))?.await?;
    println!("connection established with {:?}", conn.remote_address());

    let mut recv: iroh_net::endpoint::RecvStream = conn.accept_uni().await?;

    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        recv.read(&mut buf).await?;
        println!("{}", from_utf8(&buf)?);
    }
}
