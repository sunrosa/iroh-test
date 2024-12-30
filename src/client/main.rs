use std::str::FromStr;

use iroh::{endpoint::Connection, Endpoint, NodeAddr, PublicKey};
use iroh_test::Event;

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

    // Keep retrying connection until it's made
    let conn = loop {
        match ep.connect(addr.clone(), b"my-alpn").await {
            Ok(o) => break o,
            Err(_) => continue,
        }
    };

    // Ctrl-C handler to close connection
    tokio::task::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        ep.close().await.unwrap();
        std::process::exit(0);
    });
    {
        let conn = conn.clone();
        tokio::task::spawn(async move {
            receiver(conn)
                .await
                .unwrap_or_else(|e| panic!("{e}\n{e:?}"));
        });
    }

    let mut send = conn.open_uni().await?;
    send.write_all(&serde_json::to_vec(&Event::Connected(
        conn.stable_id().to_string(),
    ))?)
    .await?;
    send.finish()?;

    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;
        let buf = buf.trim();

        let event: Event = Event::Chat(conn.stable_id().to_string(), buf.into());

        let mut send = conn.open_uni().await?;
        send.write_all(&serde_json::to_vec(&event)?).await?;
        send.finish()?;
    }
}

async fn receiver(conn: Connection) -> anyhow::Result<()> {
    loop {
        let mut recv = conn.accept_uni().await?;

        let received = recv.read_to_end(8192).await?;
        let message: Event = serde_json::from_slice(&received)?;

        match message {
            Event::Chat(sender, content) => println!("{sender}: {content}"),
            Event::Connected(c) => println!("{c} connected."),
            Event::Disconnected(d) => println!("{d} disconnected."),
        }
    }
}
