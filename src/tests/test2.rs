use crate::u2client::client::U2client;
use crate::{u2client, Result};

#[tokio::test]
async fn test() -> Result<()> {
    let f = std::fs::read_to_string("args.toml")?;
    let args: u2client::types::Config = toml::from_str(f.as_str()).unwrap();
    let agent = U2client::new(
        &args.cookie,
        &args.passkey,
        &args.proxy,
        &args.RpcURL,
        &args.RpcUsername,
        &args.RpcPassword,
        &args.workRoot,
    )
    .await?;
    let res = agent.getTorrentInfo("14312").await?;
    println!("{:?}\n", res);
    let res = agent.getStats().await?;
    println!("{:?}\n", res);
    let res = agent.getFreeSpace("E:".to_string()).await?;
    println!("{:?}\n", res);
    Ok(())
}
