use std::str::{from_utf8, FromStr};

use anyhow::anyhow;
use iroh::{endpoint::SendStream, Endpoint, SecretKey};
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
    let (broadcastsend, broadcastrecv) = tokio::sync::broadcast::channel(1024);

    let recv1 = tokio::task::spawn(client(
        ep.clone(),
        mpscsend.clone(),
        broadcastsend.subscribe(),
        broadcastsend.clone(),
    ));
    let recv2 = tokio::task::spawn(client(ep, mpscsend, broadcastrecv, broadcastsend));

    loop {
        let msg = mpscrecv
            .recv()
            .await
            .ok_or(anyhow!("message mpsc closed and empty"))?;
        println!("{msg}");
    }

    join!(recv1, recv2);
}

async fn client(
    ep: Endpoint,
    sender: UnboundedSender<String>,
    broadcastrecv: tokio::sync::broadcast::Receiver<(usize, String)>,
    broadcastsend: tokio::sync::broadcast::Sender<(usize, String)>,
) -> anyhow::Result<()> {
    let conn = ep.accept().await.ok_or(anyhow!("err"))?.await?;
    println!("connection established with {:?}", conn.remote_address());
    let (send, mut recv) = conn.accept_bi().await?;

    tokio::task::spawn(broadcast(conn.stable_id(), send, broadcastrecv));

    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        recv.read(&mut buf).await?;

        let utf8 = from_utf8(&buf)?.trim();
        let formatted = format!("{}: {}", conn.stable_id(), utf8);

        sender.send(formatted)?;
        broadcastsend.send((conn.stable_id(), utf8.into()))?;
    }
}

async fn broadcast(
    conn_id: usize,
    mut send: SendStream,
    mut receiver: tokio::sync::broadcast::Receiver<(usize, String)>,
) -> anyhow::Result<()> {
    send.write("connected".as_bytes()).await?;

    loop {
        let msg = receiver.recv().await?;

        if conn_id != msg.0 {
            send.write(format!("{}: {}", msg.0, msg.1).as_bytes())
                .await?;
        }
    }
}
