use std::str::{from_utf8, FromStr};

use anyhow::anyhow;
use iroh::{Endpoint, SecretKey};

fn main() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    let secret_key = include_str!("secret_key").trim();

    let ep = Endpoint::builder()
        .alpns(vec![b"my-alpn".to_vec()])
        .discovery_n0()
        .secret_key(SecretKey::from_str(secret_key)?)
        .bind()
        .await?;

    let conn = ep.accept().await.ok_or(anyhow!("err"))?.await?;
    println!("connection established with {:?}", conn.remote_address());

    let mut recv = conn.accept_uni().await?;

    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        recv.read(&mut buf).await?;
        println!("{}", from_utf8(&buf)?);
    }
}
