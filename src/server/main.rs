use std::str::{from_utf8, FromStr};

use anyhow::anyhow;
use iroh::{endpoint::Connection, Endpoint, SecretKey};
use tokio::{join, sync::broadcast};

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

    let (broadcastsend, _) = broadcast::channel(1024);

    loop {
        let conn = ep.accept().await.ok_or(anyhow!("err"))?.await?;
        println!("connection established with {:?}", conn.remote_address());

        tokio::task::spawn(client(
            conn,
            broadcastsend.subscribe(),
            broadcastsend.clone(),
        ));
    }
}

async fn client(
    conn: Connection,
    broadcastrecv: broadcast::Receiver<(usize, String)>,
    broadcastsend: broadcast::Sender<(usize, String)>,
) -> anyhow::Result<()> {
    tokio::task::spawn(broadcast(conn.clone(), broadcastrecv));

    loop {
        let mut recv = conn.accept_uni().await?;
        let received = recv.read_to_end(1024).await?;
        let utf8 = from_utf8(&received)?.trim();

        broadcastsend.send((conn.stable_id(), utf8.into()))?;
        println!("{}: {}", conn.stable_id(), utf8);
    }
}

async fn broadcast(
    conn: Connection,
    mut receiver: broadcast::Receiver<(usize, String)>,
) -> anyhow::Result<()> {
    loop {
        let msg = receiver.recv().await?;

        if conn.stable_id() != msg.0 && !msg.1.is_empty() {
            let mut send = conn.open_uni().await?;
            send.write_all(format!("{}: {}", msg.0, msg.1).as_bytes())
                .await?;
            send.finish()?;
        }
    }
}
