use crate::torrentLib::request::TorrentAction;
use crate::u2client::client::U2client;
use crate::{u2client, Result};
use std::thread::sleep;
use std::time::Duration;

#[tokio::test]
async fn test() -> Result<()> {
    let f = std::fs::read_to_string("args.toml")?;
    let args: u2client::types::Config = toml::from_str(f.as_str()).unwrap();
    let agent = U2client::new(
        &args.cookie,
        &args.proxy,
        &args.RpcURL,
        &args.RpcUsername,
        &args.RpcPassword,
        &args.workRoot,
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
    sleep(Duration::from_secs(5));

    let x = agent.getRemove().await?;
    for i in x {
        let _ = agent.removeTorrent(i.hash_string.unwrap()).await?;
    }
    let res = agent.getWorkingTorrent().await?;
    println!("{:?}\n", res);
    let res = agent.getUserInfo().await?;
    println!("{:?}\n", res);

    Ok(())
}
