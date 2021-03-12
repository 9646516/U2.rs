use std::thread::sleep;
use std::time::Duration;

use crate::torrentLib::types::TorrentAction;
use crate::u2client::client::U2client;
use crate::*;

#[tokio::test]
async fn main() -> Result<()> {
    let f = std::fs::read_to_string("args.toml")?;
    let args: u2client::types::Config = toml::from_str(f.as_str()).unwrap();
    let agent = U2client::new(
        args.cookie,
        args.passkey,
        args.proxy,
        args.RpcURL,
        args.RpcUsername,
        args.RpcPassword,
        args.workRoot,
    )
    .await?;

    let res = agent.getTransmissionSession().await?;
    println!("{:?}\n", res);
    let res = agent.getWorkingTorrent().await?;
    println!("{:?}\n", res);

    let res = agent.getTorrent().await?;
    for i in 0..4 {
        let _ = agent.addTorrent(&res.get(i).unwrap().url).await?;
        println!("{} added", i);
    }
    let res = agent.getWorkingTorrent().await?;
    println!("{:?}\n", res);

    sleep(Duration::from_secs(2));
    for i in 0..4 {
        let x = res.torrents.get(i).unwrap();
        let _ = agent
            .performActionOnTorrent(x.hash_string.as_ref().unwrap().clone(), TorrentAction::Stop)
            .await?;
        println!("{} stopped", i);
    }
    sleep(Duration::from_secs(10));
    for i in 1..5 {
        let x = agent.getRemove().await?;
        let _ = agent.removeTorrent(x.hash_string.unwrap()).await?;
        println!("{} removed", i);
    }
    let res = agent.getWorkingTorrent().await?;
    println!("{:?}\n", res);
    let res = agent.getUserInfo().await?;
    println!("{:?}\n", res);
    let res = agent.getTorrentInfo("14312").await?;
    println!("{:?}\n", res);

    Ok(())
}
