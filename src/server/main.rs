use std::str::{from_utf8, FromStr};

use anyhow::anyhow;
use iroh::{Endpoint, SecretKey};
use tokio::{
    join, select,
    sync::mpsc::{self, UnboundedSender},
};

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

    let (mpscsend, mut mpscrecv) = mpsc::unbounded_channel();

    let recv1 = tokio::task::spawn(receiver(ep.clone(), mpscsend.clone()));
    let recv2 = tokio::task::spawn(receiver(ep, mpscsend));

    loop {
        let msg = mpscrecv
            .recv()
            .await
            .ok_or(anyhow!("message mpsc closed and empty"))?;
        println!("{msg}");
    }

    join!(recv1, recv2);
}

async fn receiver(ep: Endpoint, sender: UnboundedSender<String>) -> anyhow::Result<()> {
    let conn = ep.accept().await.ok_or(anyhow!("err"))?.await?;
    println!("connection established with {:?}", conn.remote_address());
    let mut recv = conn.accept_uni().await?;

    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        recv.read(&mut buf).await?;

        sender.send(format!("{}: {}", conn.stable_id(), from_utf8(&buf)?))?;
    }
}
