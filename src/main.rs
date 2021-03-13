#![allow(non_snake_case)]
#![feature(async_closure)]

use std::collections::HashSet;
use std::io::stdout;
use std::result::Result::Ok;
use std::sync::{Arc, RwLock};

use crossterm::event::{self, Event as CEvent, KeyCode};
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use sysinfo::{System, SystemExt};
use tokio::time::{sleep, Duration};
use tui::{backend::CrosstermBackend, Terminal};

use crate::u2client::client::U2client;
use crate::u2client::types::Status;
use crate::ui::TabsState;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub mod torrentLib;
pub mod u2client;
pub mod ui;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() -> Result<()> {
    let f = std::fs::read_to_string("args.toml")?;
    let args: u2client::types::Config = toml::from_str(f.as_str()).unwrap();
    let agent = Arc::from(
        U2client::new(
            &args.cookie,
            &args.passkey,
            &args.proxy,
            &args.RpcURL,
            &args.RpcUsername,
            &args.RpcPassword,
            &args.workRoot,
        )
        .await?,
    );
    let agentSep = Arc::clone(&agent);

    let lastLocal = Arc::new(RwLock::new(None));
    let lastRemote = Arc::new(RwLock::new(None));
    let mask = Arc::new(RwLock::new(0u8));

    let lastLocalSep = Arc::clone(&lastLocal);
    let lastRemoteSep = Arc::clone(&lastRemote);
    let maskSep = Arc::clone(&mask);
    let backEnd = tokio::task::spawn(async move {
        loop {
            let mut masks = 0u8;
            let remote = agentSep.getUserInfo();
            let local = agentSep.getStats();
            let (remote, local) = tokio::join!(remote, local);
            if let Ok(x) = remote {
                if let Ok(mut lastRemote) = lastRemote.write() {
                    (*lastRemote) = Some(x)
                }
                masks |= 2;
            }
            if let Ok(x) = local {
                if let Ok(mut lastLocal) = lastLocal.write() {
                    (*lastLocal) = Some(x)
                }
                masks |= 1;
            }
            if let Ok(mut mask) = mask.write() {
                (*mask) = masks;
            }
            let _ = sleep(Duration::from_millis(2000)).await;
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

    let tabStatus = Arc::new(RwLock::new(TabsState::new()));
    let tabStatusSep = Arc::clone(&tabStatus);

    let _ = enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let _ = terminal.clear()?;

    let frontEnd = tokio::task::spawn(async move {
        loop {
            let idx = match tabStatus.read() {
                Ok(tabStatus) => tabStatus.index as i8,
                _ => -1,
            };
            let masks = match maskSep.read() {
                Ok(x) => *x,
                _ => 0,
            };
            match idx {
                0 => {
                    let status = Status {
                        hardware: Some(System::new_all()),
                        local: None,
                        remote: match lastRemoteSep.read() {
                            Ok(lastRemoteSep) => lastRemoteSep.as_ref().cloned(),
                            Err(_) => None,
                        },
                    };
                    terminal.draw(|f| ui::draw(f, status, masks, 0)).unwrap();
                }
                1 => {
                    let status = Status {
                        hardware: None,
                        local: match lastLocalSep.read() {
                            Ok(lastLocalSep) => lastLocalSep.as_ref().cloned(),
                            Err(_) => None,
                        },
                        remote: None,
                    };
                    terminal.draw(|f| ui::draw(f, status, masks, 1)).unwrap();
                }
                2 => {
                    let status = Status {
                        hardware: None,
                        local: None,
                        remote: None,
                    };
                    //FIXME
                    terminal.draw(|f| ui::draw(f, status, masks, 2)).unwrap();
                }
                _ => {}
            }
            let _ = sleep(Duration::from_millis(100)).await;
        }
    });
    let keyboard = tokio::task::spawn(async move {
        let mut last_tick = std::time::Instant::now();
        let tick_rate = std::time::Duration::from_millis(100);
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    match key.code {
                        KeyCode::Char('q') => {
                            break;
                        }
                        KeyCode::Left => {
                            if let Ok(mut x) = tabStatusSep.write() {
                                (*x).previous();
                            }
                        }
                        KeyCode::Right => {
                            if let Ok(mut x) = tabStatusSep.write() {
                                (*x).next();
                            }
                        }
                        _ => {}
                    };
                }
            }
            if last_tick.elapsed() >= tick_rate {
                last_tick = std::time::Instant::now();
            }
        }
    });
    tokio::select! {
        _ = backEnd => {}
        _ = frontEnd => {}
        _ = keyboard => {}
        _ = server => {}
    }
    let _ = disable_raw_mode()?;
    Ok(())
}
