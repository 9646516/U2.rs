use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use regex::Regex;
use reqwest::IntoUrl;
use rss::Channel;
use select::document::Document;
use select::predicate::Name;

use crate::torrentLib::client::{BasicAuth, TransClient};
use crate::torrentLib::request::{Id, TorrentAction, TorrentAddArgs};
use crate::torrentLib::response::{FreeSpace, SessionGet, SessionStats, Torrent, Torrents};
use crate::u2client::types::UserInfo;
use crate::u2client::types::{RssInfo, TorrentInfo};

use super::Result;

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
        proxy: &Option<String>,
        RpcURL: &str,
        RpcUsername: &str,
        RpcPassword: &str,
        workRoot: &str,
    ) -> Result<U2client> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::COOKIE,
            format!("nexusphp_u2={}", cookie).parse()?,
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
                .ok_or("get uid failed")?
                .split('=')
                .last()
                .ok_or("get uid failed")?
                .to_string();

            let tempSpace = format!("{}/temp", workRoot);
            if !Path::new(&tempSpace).exists() {
                std::fs::create_dir(&tempSpace)?;
            }
            let workSpace = format!("{}/work", workRoot);
            if !Path::new(&workSpace).exists() {
                std::fs::create_dir(&workSpace)?;
            }
            let basic_auth = BasicAuth {
                user: RpcUsername.to_string(),
                password: RpcPassword.to_string(),
            };
            let res = container
                .post("https://u2.dmhy.org/getrss.php")
                .form(&[
                    ("inclbookmarked", 0),
                    ("inclautochecked", 1),
                    ("trackerssl", 1),
                    ("showrows", 10),
                    ("search_mode", 1),
                ])
                .send()
                .await?
                .text()
                .await?;
            let res = Document::from(res.as_str())
                .find(Name("a"))
                .find(|x| match x.attr("class") {
                    Some(str) => {
                        if str == "faqlink" {
                            match x.attr("rel") {
                                Some(str) => str == "nofollow noopener noreferer",
                                _ => false,
                            }
                        } else {
                            false
                        }
                    }
                    _ => false,
                })
                .unwrap()
                .text();
            let passkey = U2client::matchRegex(&res, "passkey=([0-9a-z]*)")?;
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
            .ok_or("addTorrent:can not find content-disposition header")?
            .to_str()?;
        let filename = U2client::matchRegex(contentDisposition, "filename=%5BU2%5D.(.+)")?;
        let to = format!("{}/{}", self.tempSpace, filename);
        let toPath = Path::new(&to);
        let content = s.bytes().await?;
        if toPath.exists() {
            std::fs::remove_file(&toPath)?;
        }
        let mut file = std::fs::File::create(&toPath)?;
        file.write_all(&*content)?;

        let add: TorrentAddArgs = TorrentAddArgs {
            filename: Some(to),
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
        Ok(self.getTorrent().await?)
    }

    pub async fn getRemove(&self) -> Result<Vec<Torrent>> {
        let mut torrent = self.getWorkingTorrent().await?;
        torrent.torrents.sort_by_key(|x| {
            (
                x.peers_getting_from_us.unwrap_or(0),
                x.added_date.unwrap_or(0),
            )
        });
        Ok(torrent.torrents.into_iter().take(5).collect())
    }

    pub async fn getUserInfo(&self) -> Result<UserInfo> {
        let context = self
            .get(format!(
                "https://u2.dmhy.org/userdetails.php?id={}",
                self.uid
            ))
            .await?;

        let username = Document::from(context.as_str())
            .find(Name("a"))
            .find(|x| match x.attr("class") {
                Some(x) => x == "User_Name",
                _ => false,
            })
            .ok_or("getUserInfo:can not find username node")?
            .text();

        let body: HashMap<String, String> = U2client::parseHtml(&context, 2)?;

        let t = U2client::reduceToText(&body, "BT时间")?;
        let timeRate = U2client::matchRegex(&t, "做种/下载时间比率:[' ']*([0-9.]+)")?;
        let uploadTime = U2client::matchRegex(&t, "做种时间:[' ']*([天0-9:' ']+[0-9])")?;
        let downloadTime = U2client::matchRegex(&t, "下载时间:[' ']*([天0-9:' ']+[0-9])")?;

        let t = U2client::reduceToText(&body, "传输[历史]")?;
        let shareRate = U2client::matchRegex(&t, "分享率:[' ']*([0-9.]+)")?;
        let upload = U2client::matchRegex(&t, "上传量:[' ']*([0-9.' ']+[TGMK]iB)")?;
        let download = U2client::matchRegex(&t, "下载量:[' ']*([0-9.' ']+[TGMK]iB)")?;
        let actualUpload = U2client::matchRegex(&t, "实际上传:[' ']*([0-9.' ']+[TGMK]iB)")?;
        let actualDownload = U2client::matchRegex(&t, "实际下载:[' ']*([0-9.' ']+[TGMK]iB)")?;

        let t = U2client::reduceToText(&body, "UCoin[详情]")?;
        let coin = U2client::matchRegex(&t, "[(]([0-9.,]+)[)]")?;

        Ok(UserInfo {
            username,
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
    /// 2 => Free
    /// 3 => 2x
    /// 4 => 2xFree
    /// 5 => 50%off
    /// 6 => 2x50%off
    /// 7 => 30%off
    pub async fn applyMagic(&self, uid: &str, time: i32, magic: i32) -> Result<()> {
        let time = time.max(24);
        let url = format!(
            "https://u2.dmhy.org/promotion.php?action=magic&torrent={}",
            uid
        );
        let post = [
            ("action", "magic".to_string()),
            ("torrent", uid.to_string()),
            ("user", "SELF".to_string()),
            ("hours", time.to_string()),
            ("promotion", magic.to_string()),
        ];
        let res = self.container.post(&url).form(&post).send().await?;
        if res.status().as_u16() == 200 {
            Ok(())
        } else {
            Err("apply magic failed:network failed".into())
        }
    }
    pub async fn getTorrent(&self) -> Result<Vec<RssInfo>> {
        let url = format!(
            "https://u2.dmhy.org/torrentrss.php?rows=50&trackerssl=1&passkey={}",
            self.passkey
        );
        let content = self.get(url).await?.into_bytes();
        let channel = Channel::read_from(&content[..])?;
        let res = channel.items.iter().map(async move |x| -> Result<RssInfo> {
            let title = x.title.clone().ok_or("getTorrent:bad rss feed")?;
            let url = x.enclosure.clone().ok_or("getTorrent:bad rss feed")?.url;
            let cat = x.categories[0].name.clone();
            let uid = U2client::matchRegex(url.as_str(), "id=([0-9]+)")?;
            let U2Info = self.getTorrentInfo(&uid).await?;
            Ok(RssInfo {
                title,
                url,
                cat,
                uid,
                U2Info,
            })
        });

        let res: Vec<Result<RssInfo>> = futures::future::join_all(res).await;
        let mut ret = Vec::new();
        for x in res.into_iter() {
            ret.push(x?);
        }
        Ok(ret)
    }
    pub async fn getTorrentInfo(&self, idx: &str) -> Result<TorrentInfo> {
        let toNumber = |x: &str| -> Result<f32> {
            Ok(U2client::matchRegex(&x.to_string(), "([0-9.]+)")?.parse::<f32>()?)
        };
        let context = self
            .get(format!("https://u2.dmhy.org/details.php?id={}", idx))
            .await?;
        let body: HashMap<String, String> = U2client::parseHtml(&context, 1)?;

        let doc = Document::from(
            body.get("流量优惠")
                .ok_or("getTorrentInfo:bad html")?
                .as_str(),
        );
        let sink = doc
            .find(select::predicate::Any)
            .next()
            .ok_or("getTorrentInfo:can find main table")?;

        let typeNode = sink.find(Name("img")).next();
        let (uploadFX, downloadFX) = if let Some(typeNode) = typeNode {
            let typeNode = typeNode
                .attr("alt")
                .ok_or("getTorrentInfo:can find alt for fx")?;
            match typeNode {
                "FREE" => (1.0, 0.0),
                "2X Free" => (2.0, 0.0),
                "30%" => (1.0, 0.3),
                "2X 50%" => (2.0, 0.5),
                "50%" => (1.0, 0.5),
                "2X" => (2.0, 1.0),
                "Promotion" => {
                    let mut iters = sink.find(Name("b"));

                    let f = toNumber(
                        &*iters
                            .next()
                            .ok_or("getTorrentInfo:can find promotion")?
                            .text(),
                    )?;
                    let s = toNumber(
                        &*iters
                            .next()
                            .ok_or("getTorrentInfo:can find promotion")?
                            .text(),
                    )?;
                    (f, s)
                }
                _ => (1.0, 1.0),
            }
        } else {
            (1.0, 1.0)
        };

        let s = U2client::reduceToText(&body, "基本信息")?;
        let size = U2client::matchRegex(&s, "大小:[' ']*([0-9.' ']+[TGMK]iB)")?;
        let number = toNumber(&*size)?;
        let GbSize = match size
            .chars()
            .nth(size.len() - 3)
            .ok_or("getTorrentInfo:bad torrent size")?
        {
            'T' => number * 1024.0,
            'G' => number,
            'M' => number / 1024.0,
            _ => number / 1024.0 / 1024.0,
        };

        let s = U2client::reduceToText(&body, "同伴[查看列表][隐藏列表]")?;
        let seeder = U2client::matchRegex(&s, "([0-9]+)[' ']*个做种者")?.parse::<i32>()?;
        let leecher = U2client::matchRegex(&s, "([0-9]+)[' ']*个下载者")?.parse::<i32>()?;

        let s = U2client::reduceToText(&body, "活力度")?;
        let avgProgress = U2client::matchRegex(&s, "平均进度:[' ']*[(]([0-9]+%)[)]")
            .unwrap_or_else(|_| String::from("100%"));
        let avgProgress = toNumber(&avgProgress)? / 100.0;

        let s = U2client::reduceToText(&body, "种子信息")?;
        let Hash = U2client::matchRegex(&s, "种子散列值:[' ']*([0-9a-z]*)[' ']*")?;
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

    fn matchRegex(src: &str, reg: &str) -> Result<String> {
        Ok(Regex::new(reg)?
            .captures_iter(src)
            .next()
            .ok_or("matchRegex:regex match failed")?
            .get(1)
            .ok_or("matchRegex:regex match failed")?
            .as_str()
            .to_string())
    }

    fn reduceToText(mp: &HashMap<String, String>, idx: &str) -> Result<String> {
        let str = mp.get(idx).ok_or("reduceToText:broken html")?.as_str();
        let ret = Document::from(str)
            .find(select::predicate::Any)
            .next()
            .ok_or("reduceToText:can not find Any Node")?
            .text();
        Ok(Regex::new("([\u{00ad}\u{00a0}])")?
            .replace_all(&*ret, "")
            .to_string())
    }
    fn parseHtml(context: &str, timesOfReduce: i32) -> Result<HashMap<String, String>> {
        let doc = Document::from(context);
        let mut outer = doc
            .find(Name("td"))
            .find(|x| match x.attr("class") {
                Some(x) => x == "outer",
                _ => false,
            })
            .ok_or("parseHtml:parse failed")?;
        for _ in 0..timesOfReduce {
            outer = outer
                .find(Name("tbody"))
                .next()
                .ok_or("parseHtml:reduce failed")?;
        }
        Ok(outer
            .children()
            .filter_map(|x| {
                let mut V = Vec::new();
                for i in x.children() {
                    let s = i.text();
                    if s.len() == 1 && s.into_bytes().get(0).unwrap_or(&b'\n') == &b'\n' {
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
            .collect())
    }
}
