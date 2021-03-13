#![allow(non_snake_case)]
#![feature(async_closure)]

use std::collections::HashSet;
use std::io::stdout;
use std::result::Result::Ok;

use tokio::time::{Duration, sleep};
use tui::{backend::CrosstermBackend, Terminal};

use crate::u2client::client::U2client;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub mod UI;
pub mod torrentLib;
pub mod u2client;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() -> Result<()> {
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

    let UI = tokio::task::spawn(async {
        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.clear().unwrap();
        let handleOne = async || -> Result<()> { Ok(()) };
        loop {
            match handleOne().await {
                Ok(_) => {}
                Err(x) => {
                    panic!("{}", x);
                }
            }
            terminal.draw(|f| UI::draw(f)).unwrap();
            let _ = sleep(Duration::from_millis(200)).await;
        }
    });

    let server = tokio::task::spawn(async move {
        let handleOne = async || -> Result<()> {
            let torrentList = agent
                .getWorkingTorrent()
                .await?
                .torrents
                .iter()
                .map(|x| x.hash_string.as_ref().unwrap().clone())
                .collect::<HashSet<String>>();
            let feed = agent.getDownloadList().await?;

            let work = feed.iter().filter_map(|i| {
                if !torrentList.contains(&i.url) {
                    Some(agent.addTorrent(&i.url))
                } else {
                    None
                }
            });

            let _ = futures::future::join_all(work).await;
            Ok(())
        };
        loop {
            match handleOne().await {
                Ok(_) => {}
                Err(x) => {
                    panic!("{}", x);
                }
            }
            let _ = sleep(Duration::from_secs(4)).await;
        }
    });
    let _ = tokio::join!(UI, server);
    Ok(())
}
