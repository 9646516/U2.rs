#![allow(non_snake_case)]
#![feature(async_closure)]

use std::collections::HashSet;
use std::io::{stdout, Stdout};
use std::path::Path;
use std::result::Result::Ok;
use std::sync::{Arc, RwLock};

use crossterm::event::{self, Event as CEvent, KeyCode};
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use log::error;
use log::info;
use log::LevelFilter;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::{
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
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
    let args: u2client::types::Config = toml::from_str(f.as_str())?;
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

    let root = args.LogRoot.to_owned();
    let rootPath = Path::new(&root);

    if !rootPath.exists() {
        std::fs::create_dir(rootPath)?;
    }
    if !rootPath.is_dir() {
        panic!("wrong log root {}, expect a directory", root);
    }

    let mainDir = format!("{}/main.log", root);
    let archivedDir = format!("{}/{}.log", root, "{}");

    let roller = FixedWindowRoller::builder().build(&archivedDir, 1)?;
    let trigger = SizeTrigger::new(100_000);
    let policy = CompoundPolicy::new(Box::new(trigger), Box::new(roller));

    let appender = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} - {m}\n",
        )))
        .build(&mainDir, Box::new(policy))?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(appender)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    let _ = log4rs::init_config(config)?;

    let lastLocal = Arc::new(RwLock::new(None));
    let lastRemote = Arc::new(RwLock::new(None));
    let mask = Arc::new(RwLock::new(0u8));
    let tabStatus = Arc::new(RwLock::new(TabsState::new()));

    let agentSep = Arc::clone(&agent);
    let agentSep2 = Arc::clone(&agent);
    let lastLocalSep = Arc::clone(&lastLocal);
    let lastRemoteSep = Arc::clone(&lastRemote);
    let maskSep = Arc::clone(&mask);
    let tabStatusSep = Arc::clone(&tabStatus);

    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    info!("init done");

    let promote = tokio::task::spawn(async move {
        let handleOne = async || -> Result<()> {
            let mut torrentList: HashSet<String> = HashSet::new();
            let working = agent.getWorkingTorrent().await?.torrents;
            for x in working.into_iter() {
                let x = x.hash_string.ok_or("handleOne:bad torrent hash")?;
                torrentList.insert(x);
            }
            let feed = agent.getDownloadList().await?;

            let work = feed.iter().filter_map(|i| {
                if !torrentList.contains(&i.url)
                    && i.U2Info.downloadFX == 0.0
                    && i.U2Info.avgProgress < 0.3
                    && i.U2Info.seeder > 0
                {
                    info!(
                        "promote:new job:{},{},{} GB",
                        &i.title, &i.cat, &i.U2Info.GbSize
                    );
                    Some(agent.addTorrent(&i.url))
                } else {
                    None
                }
            });

            let res = futures::future::join_all(work).await;
            for i in res.into_iter() {
                let _ = i?;
            }
            Ok(())
        };
        loop {
            match handleOne().await {
                Ok(_) => {}
                Err(x) => {
                    error!("promote:{}", x);
                }
            }
            sleep(Duration::from_secs(4)).await;
        }
    });

    let backEnd = tokio::task::spawn(async move {
        loop {
            let mut masks = 0u8;
            let remote = agentSep.getUserInfo();
            let local = agentSep.getStats();
            let (remote, local) = tokio::join!(remote, local);
            if let Ok(x) = remote {
                if let Ok(mut lastRemote) = lastRemote.write() {
                    *lastRemote = Some(x)
                } else {
                    error!("backEnd:get remote lock failed");
                }
                masks |= 2;
            } else {
                error!("backEnd:get U2 Info failed");
            }

            if let Ok(x) = local {
                if let Ok(mut lastLocal) = lastLocal.write() {
                    *lastLocal = Some(x)
                } else {
                    error!("backEnd:get local lock failed");
                }
                masks |= 1;
            } else {
                error!("backEnd:get BT local info failed");
            }
            if let Ok(mut mask) = mask.write() {
                *mask = masks;
            } else {
                error!("backEnd:get masks lock failed");
            }
            sleep(Duration::from_secs(2)).await;
        }
    });

    let frontEnd = tokio::task::spawn(async move {
        let handleOne = |T: &mut Terminal<CrosstermBackend<Stdout>>| -> Result<()> {
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
                        logDir: None,
                    };
                    T.draw(|f| ui::draw(f, status, masks, 0))?;
                }
                1 => {
                    let status = Status {
                        hardware: None,
                        local: match lastLocalSep.read() {
                            Ok(lastLocalSep) => lastLocalSep.as_ref().cloned(),
                            Err(_) => None,
                        },
                        remote: None,
                        logDir: None,
                    };
                    T.draw(|f| ui::draw(f, status, masks, 1))?;
                }
                2 => {
                    let status = Status {
                        hardware: None,
                        local: None,
                        remote: None,
                        logDir: Some(mainDir.to_owned()),
                    };
                    T.draw(|f| ui::draw(f, status, masks, 2))?;
                }
                _ => {}
            }
            Ok(())
        };
        loop {
            match handleOne(&mut terminal) {
                Ok(_) => {}
                Err(x) => {
                    error!("frontEnd:{}", x);
                }
            }
            sleep(Duration::from_millis(50)).await;
        }
    });

    let keyboard = tokio::task::spawn(async move {
        let mut last_tick = std::time::Instant::now();
        let tick_rate = std::time::Duration::from_millis(50);
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
    let MAX_SIZE = args.maxSize;

    let maintain = tokio::task::spawn(async move {
        let handleOne = async || -> Result<()> {
            let now = agentSep2.getWorkingTorrent().await?.torrents;
            let mut tot = 0f32;
            for i in now.into_iter() {
                let sb = i.total_size.unwrap_or(0) as f32 / 1e9;
                tot += sb;
            }
            if tot > MAX_SIZE {
                let V = agentSep2.getRemove().await?;
                let mut all = Vec::new();
                for i in V.into_iter() {
                    info!(
                        "maintain:remove {}, {} GB",
                        i.name.ok_or("handleOne:broken name")?,
                        i.total_size.ok_or("handleOne:broken size")? as f32 / 1e9
                    );
                    let hash = i.hash_string.ok_or("handleOne:broken hash")?;
                    all.push(agentSep2.removeTorrent(hash));
                }
                let res = futures::future::join_all(all).await;
                for i in res.into_iter() {
                    let _ = i?;
                }
                info!("maintain:del done");
            }
            Ok(())
        };
        loop {
            match handleOne().await {
                Ok(_) => {}
                Err(x) => {
                    error!("maintain:{}", x);
                }
            }
            sleep(Duration::from_secs(60 * 10)).await;
        }
    });
    let _ = tokio::select! {
        _ = promote => {}
        _ = backEnd => {}
        _ = frontEnd => {}
        _ = keyboard => {}
        _ = maintain => {}
    };
    disable_raw_mode()?;
    Ok(())
}
