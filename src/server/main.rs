use std::{
    str::{from_utf8, FromStr},
    time::Duration,
};

use anyhow::anyhow;
use iroh::{
    endpoint::{Connection, ConnectionError},
    Endpoint, SecretKey,
};
use tokio::sync::broadcast;

use iroh_test::Event;

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

    let (tx, _) = broadcast::channel(1024);

    loop {
        let conn = ep.accept().await.ok_or(anyhow!("err"))?.await?;
        println!("connection established with {:?}", conn.remote_address());

        tokio::task::spawn(client_handler(conn, tx.subscribe(), tx.clone()));
    }
}

async fn client_handler(
    conn: Connection,
    rx: broadcast::Receiver<(usize, Event)>,
    tx: broadcast::Sender<(usize, Event)>,
) {
    // Panic when Try unwinding
    client(conn, rx, tx)
        .await
        .unwrap_or_else(|e| panic!("{e}\n{e:?}"));
}

async fn client(
    conn: Connection,
    rx: broadcast::Receiver<(usize, Event)>,
    tx: broadcast::Sender<(usize, Event)>,
) -> anyhow::Result<()> {
    tokio::task::spawn(broadcast(conn.clone(), rx));

    loop {
        let mut recv = if let Ok(o) =
            tokio::time::timeout(Duration::from_secs(30), conn.accept_uni()).await
        {
            match o {
                Ok(o) => o,
                Err(
                    ConnectionError::ApplicationClosed(_) | ConnectionError::ConnectionClosed(_),
                ) => {
                    println!("{}: disconnected", conn.stable_id());
                    tx.send((
                        conn.stable_id(),
                        Event::Disconnected(conn.stable_id().to_string()),
                    ))?; // Broadcast to *other* clients
                    break Ok(());
                }
                Err(e) => Err(e)?,
            }
        } else {
            if conn.close_reason().is_some() {
                println!("{}: disconnected", conn.stable_id());
                tx.send((
                    conn.stable_id(),
                    Event::Disconnected(conn.stable_id().to_string()),
                ))?; // Broadcast to *other* clients
                break Ok(());
            }

            continue;
        };

        let received = recv.read_to_end(8192).await?;

        let event: Event = serde_json::from_slice(&received)?;

        match event {
            Event::Chat(sender, content) => {
                tx.send((conn.stable_id(), Event::Chat(sender, content.clone())))?;
            }
            Event::Connected(c) => {
                tx.send((conn.stable_id(), Event::Connected(c)))?;
            }
            Event::Disconnected(c) => {
                tx.send((conn.stable_id(), Event::Disconnected(c)))?;
            }
        }
    }
}

async fn broadcast(
    conn: Connection,
    mut receiver: broadcast::Receiver<(usize, Event)>,
) -> anyhow::Result<()> {
    loop {
        let msg = receiver.recv().await?;

        if conn.stable_id() != msg.0 {
            let mut send = conn.open_uni().await?;
            send.write_all(&serde_json::to_vec(&msg.1)?).await?;
            send.finish()?;
        }
    }
}
