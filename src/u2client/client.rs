use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use regex::Regex;
use reqwest::IntoUrl;
use rss::Channel;
use select::document::Document;
use select::predicate::Name;

use crate::torrentLib::client::{BasicAuth, TransClient};
use crate::u2client::types::UserInfo;
use crate::u2client::types::{RssInfo, TorrentInfo};

use super::Result;
use crate::torrentLib::request::{Id, TorrentAction, TorrentAddArgs};
use crate::torrentLib::response::{FreeSpace, SessionGet, SessionStats, Torrent, Torrents};

#[derive(Clone)]
pub struct U2client {
    uid: String,
    passkey: String,
    container: reqwest::Client,
    torrentClient: TransClient,
    tempSpace: String,
    workSpace: String,
}

impl U2client {
    pub async fn new(
        cookie: &str,
        passkey: &str,
        proxy: &Option<String>,
        RpcURL: &str,
        RpcUsername: &str,
        RpcPassword: &str,
        workRoot: &str,
    ) -> Result<U2client> {
        let passkey = passkey.to_string();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::COOKIE,
            format!("nexusphp_u2={}", cookie).parse().unwrap(),
        );

        let mut container = reqwest::Client::builder()
            .cookie_store(true)
            .default_headers(headers);

        if let Some(ref x) = proxy {
            let proxy = reqwest::Proxy::http(x)?;
            container = container.proxy(proxy);

            let proxy = reqwest::Proxy::https(x)?;
            container = container.proxy(proxy);
        }

        let container = container.build()?;

        let x = container
            .get("https://u2.dmhy.org/index.php")
            .send()
            .await?;

        if x.url().path() == "/index.php" {
            let context = x.text().await?;
            let uid = Document::from(context.as_str())
                .find(Name("a"))
                .filter(|x| match x.attr("class") {
                    Some(x) => x == "User_Name",
                    _ => false,
                })
                .filter_map(|n| n.attr("href"))
                .map(|x| x.to_string())
                .next()
                .unwrap()
                .split('=')
                .last()
                .unwrap()
                .to_string();

            let tempSpace = format!("{}/temp", workRoot);
            if !Path::new(&tempSpace).exists() {
                std::fs::create_dir(&tempSpace).unwrap();
            }
            let workSpace = format!("{}/work", workRoot);
            if !Path::new(&workSpace).exists() {
                std::fs::create_dir(&workSpace).unwrap();
            }
            let basic_auth = BasicAuth {
                user: RpcUsername.to_string(),
                password: RpcPassword.to_string(),
            };

            Ok(U2client {
                uid,
                passkey,
                container,
                torrentClient: TransClient::with_auth(&RpcURL, basic_auth),
                tempSpace,
                workSpace,
            })
        } else {
            Err("illegal cookie".into())
        }
    }
    pub async fn removeTorrent(&self, id: String) -> Result<()> {
        let _ = self
            .torrentClient
            .torrent_remove(vec![Id::Hash(id)], true)
            .await?;
        Ok(())
    }
    pub async fn addTorrent(&self, url: &str) -> Result<()> {
        let s = self.container.get(url).send().await?;
        let contentDisposition = s
            .headers()
            .get("content-disposition")
            .unwrap()
            .to_str()
            .unwrap();
        let filename = U2client::matchRegex(contentDisposition, "filename=%5BU2%5D.(.+)").unwrap();
        let to = format!("{}/{}", self.tempSpace, filename);
        let to = Path::new(to.as_str());
        let content = s.bytes().await?;
        if to.exists() {
            let _ = std::fs::remove_file(to);
        }
        let mut file = std::fs::File::create(to)?;
        file.write_all(&*content)?;

        let add: TorrentAddArgs = TorrentAddArgs {
            filename: Some(to.to_str().unwrap().to_string()),
            download_dir: Some(self.workSpace.clone()),
            ..TorrentAddArgs::default()
        };
        let _ = self.torrentClient.torrent_add(add).await?;
        Ok(())
    }
    pub async fn getTransmissionSession(&self) -> Result<SessionGet> {
        Ok(self.torrentClient.session_get().await?.arguments)
    }

    pub async fn performActionOnTorrent(&self, id: String, op: TorrentAction) -> Result<()> {
        let _ = self
            .torrentClient
            .torrent_action(op, vec![Id::Hash(id)])
            .await?;
        Ok(())
    }

    pub async fn getWorkingTorrent(&self) -> Result<Torrents<Torrent>> {
        Ok(self.torrentClient.torrent_get(None, None).await?.arguments)
    }

    pub async fn getStats(&self) -> Result<SessionStats> {
        Ok(self.torrentClient.session_stats().await?.arguments)
    }

    pub async fn getFreeSpace(&self, d: String) -> Result<FreeSpace> {
        Ok(self.torrentClient.free_space(d).await?.arguments)
    }

    pub async fn getDownloadList(&self) -> Result<Vec<RssInfo>> {
        let rss = self.getTorrent().await?;
        Ok(rss
            .into_iter()
            .filter(|x| {
                x.U2Info.downloadFX == 0.0 && x.U2Info.avgProgress < 0.3 && x.U2Info.seeder > 0
            })
            .collect())
    }

    pub async fn getRemove(&self) -> Result<Vec<Torrent>> {
        let mut torrent = self.getWorkingTorrent().await?;
        torrent
            .torrents
            .sort_by_key(|x| (x.peers_getting_from_us.unwrap(), x.added_date.unwrap()));
        Ok(torrent.torrents.into_iter().take(5).collect())
    }

    pub async fn getUserInfo(&self) -> Result<UserInfo> {
        let context = self
            .get(format!(
                "https://u2.dmhy.org/userdetails.php?id={}",
                self.uid
            ))
            .await?;
        let body: HashMap<String, String> = U2client::parseHtml(&context, 2);

        let t = U2client::reduceToText(&body, "BT时间");
        let timeRate = U2client::matchRegex(&t, "做种/下载时间比率:[' ']*([0-9.]+)").unwrap();
        let uploadTime = U2client::matchRegex(&t, "做种时间:[' ']*([天0-9:' ']+[0-9])").unwrap();
        let downloadTime = U2client::matchRegex(&t, "下载时间:[' ']*([天0-9:' ']+[0-9])").unwrap();

        let t = U2client::reduceToText(&body, "传输[历史]");
        let shareRate = U2client::matchRegex(&t, "分享率:[' ']*([0-9.]+)").unwrap();
        let upload = U2client::matchRegex(&t, "上传量:[' ']*([0-9.' ']+[TGMK]iB)").unwrap();
        let download = U2client::matchRegex(&t, "下载量:[' ']*([0-9.' ']+[TGMK]iB)").unwrap();
        let actualUpload = U2client::matchRegex(&t, "实际上传:[' ']*([0-9.' ']+[TGMK]iB)").unwrap();
        let actualDownload =
            U2client::matchRegex(&t, "实际下载:[' ']*([0-9.' ']+[TGMK]iB)").unwrap();

        let t = U2client::reduceToText(&body, "UCoin[详情]");
        let coin = U2client::matchRegex(&t, "[(]([0-9.,]+)[)]").unwrap();

        Ok(UserInfo {
            download,
            upload,
            shareRate,

            actualDownload,
            actualUpload,

            coin,

            downloadTime,
            uploadTime,
            timeRate,
        })
    }

    pub async fn getTorrent(&self) -> Result<Vec<RssInfo>> {
        let url = format!(
            "https://u2.dmhy.org/torrentrss.php?rows=50&trackerssl=1&passkey={}",
            self.passkey
        );
        let content = self.get(url).await?.into_bytes();
        let channel = Channel::read_from(&content[..])?;
        let res = channel.items.iter().map(async move |x| -> Result<RssInfo> {
            let title = x.title.clone().unwrap();
            let url = x.enclosure.clone().unwrap().url;
            let cat = x.categories[0].name.clone();
            let uid = U2client::matchRegex(url.as_str(), "id=([0-9]+)").unwrap();
            let U2Info = self.getTorrentInfo(&uid).await?;
            Ok(RssInfo {
                title,
                url,
                cat,
                U2Info,
            })
        });
        let res: Vec<Result<RssInfo>> = futures::future::join_all(res).await;
        let res = res.iter().map(|x| x.as_ref().unwrap().clone()).collect();
        Ok(res)
    }
    pub async fn getTorrentInfo(&self, idx: &str) -> Result<TorrentInfo> {
        let toNumber = |x: &str| -> Result<f32> {
            Ok(U2client::matchRegex(&x.to_string(), "([0-9.]+)")
                .unwrap()
                .parse::<f32>()?)
        };
        let context = self
            .get(format!("https://u2.dmhy.org/details.php?id={}", idx))
            .await?;
        let body: HashMap<String, String> = U2client::parseHtml(&context, 1);

        let doc = Document::from(body.get("流量优惠").unwrap().as_str());
        let sink = doc.find(select::predicate::Any).next().unwrap();

        let typeNode = sink.find(Name("img")).next();
        let (uploadFX, downloadFX) = if let Some(typeNode) = typeNode {
            let typeNode = typeNode.attr("alt").unwrap();
            match typeNode {
                "FREE" => (1.0, 0.0),
                "2X Free" => (2.0, 0.0),
                "30%" => (1.0, 0.3),
                "2X 50%" => (2.0, 0.5),
                "50%" => (1.0, 0.5),
                "2X" => (2.0, 1.0),
                "Promotion" => {
                    let mut iters = sink.find(Name("b"));

                    let f = toNumber(&*iters.next().unwrap().text())?;
                    let s = toNumber(&*iters.next().unwrap().text())?;
                    (f, s)
                }
                _ => (1.0, 1.0),
            }
        } else {
            (1.0, 1.0)
        };

        let s = U2client::reduceToText(&body, "基本信息");
        let size = U2client::matchRegex(&s, "大小:[' ']*([0-9.' ']+[TGMK]iB)").unwrap();
        let number = toNumber(&*size)?;
        let GbSize = match size.chars().nth(size.len() - 3).unwrap() {
            'T' => number * 1024.0,
            'G' => number,
            'M' => number / 1024.0,
            _ => number / 1024.0 / 1024.0,
        };

        let s = U2client::reduceToText(&body, "同伴[查看列表][隐藏列表]");
        let seeder = U2client::matchRegex(&s, "([0-9]+)[' ']*个做种者")
            .unwrap()
            .parse::<i32>()?;
        let leecher = U2client::matchRegex(&s, "([0-9]+)[' ']*个下载者")
            .unwrap()
            .parse::<i32>()?;

        let s = U2client::reduceToText(&body, "活力度");
        let avgProgress = U2client::matchRegex(&s, "平均进度:[' ']*[(]([0-9]+%)[)]")
            .unwrap_or_else(|| String::from("0%"));
        let avgProgress = toNumber(&avgProgress)? / 100.0;

        let s = U2client::reduceToText(&body, "种子信息");
        let Hash = U2client::matchRegex(&s, "种子散列值:[' ']*([0-9a-z]*)[' ']*").unwrap();
        Ok(TorrentInfo {
            GbSize,
            uploadFX,
            downloadFX,
            seeder,
            leecher,
            avgProgress,
            Hash,
        })
    }
    async fn get<T>(&self, url: T) -> Result<String>
    where
        T: IntoUrl,
    {
        let ret = self.container.get(url).send().await?;
        if ret.status().as_u16() == 200 {
            Ok(ret.text().await?)
        } else {
            Err(ret.text().await?.into())
        }
    }

    fn matchRegex(src: &str, reg: &str) -> Option<String> {
        Some(
            Regex::new(reg)
                .unwrap()
                .captures_iter(src)
                .next()?
                .get(1)?
                .as_str()
                .to_string(),
        )
    }

    fn reduceToText(mp: &HashMap<String, String>, idx: &str) -> String {
        let ret = Document::from(mp.get(idx).unwrap().as_str())
            .find(select::predicate::Any)
            .next()
            .unwrap()
            .text();
        Regex::new("([\u{00ad}\u{00a0}])")
            .unwrap()
            .replace_all(&*ret, "")
            .to_string()
    }
    fn parseHtml(context: &str, timesOfReduce: i32) -> HashMap<String, String> {
        let doc = Document::from(context);
        let mut outer = doc
            .find(Name("td"))
            .find(|x| match x.attr("class") {
                Some(x) => x == "outer",
                _ => false,
            })
            .unwrap();
        for _ in 0..timesOfReduce {
            outer = outer.find(Name("tbody")).next().unwrap();
        }
        outer
            .children()
            .filter_map(|x| {
                let mut V = Vec::new();
                for i in x.children() {
                    let s = i.text();
                    if s.len() == 1 && *s.into_bytes().get(0).unwrap() == b'\n' {
                        continue;
                    } else {
                        V.push(i);
                    }
                }
                if V.len() == 2 {
                    Some((V[0].text(), V[1].html()))
                } else {
                    None
                }
            })
            .collect()
    }
}
