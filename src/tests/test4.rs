use crate::u2client::client::U2client;
use crate::{u2client, Result};

#[tokio::test]
async fn test() -> Result<()> {
    let f = std::fs::read_to_string("args.toml").expect("can not find args.toml");
    let args: u2client::types::Config = toml::from_str(f.as_str()).expect("wrong toml format");
    let agent = U2client::new(
        &args.cookie,
        &args.proxy,
        &args.RpcURL,
        &args.RpcUsername,
        &args.RpcPassword,
        &args.workRoot,
    )
    .await?;
    let _ = agent.applyMagic("234", 24, 5).await;
    Ok(())
}
