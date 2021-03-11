use std::collections::HashMap;

use regex::Regex;
use reqwest::IntoUrl;
use select::document::Document;
use select::predicate::Name;

use crate::u2client::types::TorrentInfo;
use crate::u2client::types::UserInfo;
use crate::u2client::Result;

pub struct U2client {
    uid: String,
    container: reqwest::Client,
}

impl U2client {
    pub async fn new(cookie: &str, proxy: Option<&str>) -> Result<U2client> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::COOKIE,
            format!("nexusphp_u2={}", cookie).parse().unwrap(),
        );

        let mut container = reqwest::Client::builder()
            .cookie_store(true)
            .default_headers(headers);

        if let Some(x) = proxy {
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
            Ok(U2client { uid, container })
        } else {
            Err("illegal cookie".into())
        }
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
        let timeRate = U2client::matchRegex(&t, "做种/下载时间比率:[' ']*([0-9.]+)");
        let uploadTime = U2client::matchRegex(&t, "做种时间:[' ']*([天0-9:' ']+[0-9])");
        let downloadTime = U2client::matchRegex(&t, "下载时间:[' ']*([天0-9:' ']+[0-9])");

        let t = U2client::reduceToText(&body, "传输[历史]");
        let shareRate = U2client::matchRegex(&t, "分享率:[' ']*([0-9.]+)");
        let upload = U2client::matchRegex(&t, "上传量:[' ']*([0-9.' ']+[TGMK]iB)");
        let download = U2client::matchRegex(&t, "下载量:[' ']*([0-9.' ']+[TGMK]iB)");
        let actualUpload = U2client::matchRegex(&t, "实际上传:[' ']*([0-9.' ']+[TGMK]iB)");
        let actualDownload = U2client::matchRegex(&t, "实际下载:[' ']*([0-9.' ']+[TGMK]iB)");

        let t = U2client::reduceToText(&body, "UCoin[详情]");
        let coin = U2client::matchRegex(&t, "[(]([0-9.,]+)[)]");

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

    pub async fn getTorrent(&self) {}
    pub async fn getTorrentInfo(&self, idx: String) -> Result<TorrentInfo> {
        let toNumber = |x: &str| {
            let x = U2client::matchRegex(&x.to_string(), "([0-9.]+)");
            let x = x.as_bytes();
            let len = x.len();
            let mut f = 0.0;
            let mut s = 0.0;
            for i in 0..len {
                if *x.get(i).unwrap() == b'.' {
                    break;
                } else {
                    f = f * 10.0 + (x.get(i).unwrap() - b'0') as f32;
                }
            }
            for i in (0..len).rev() {
                if *x.get(i).unwrap() == b'.' {
                    break;
                } else {
                    s = s * 0.1 + (x.get(i).unwrap() - b'0') as f32;
                }
            }
            f + s * 0.1
        };
        let context = self
            .get(format!("https://u2.dmhy.org/details.php?id={}", idx))
            .await?;
        let body: HashMap<String, String> = U2client::parseHtml(&context, 1);

        let doc = Document::from(body.get("流量优惠").unwrap().as_str());
        let sink = doc.find(select::predicate::Any).next().unwrap();

        let typeNode = sink.find(Name("img")).next().unwrap().attr("alt").unwrap();
        let (uploadFX, downloadFX) = match typeNode {
            "FREE" => (1.0, 0.0),
            "2X Free" => (2.0, 0.0),
            "30%" => (1.0, 0.3),
            "2X 50%" => (2.0, 0.5),
            "50%" => (1.0, 0.5),
            "2X" => (2.0, 1.0),
            "Promotion" => {
                let mut iters = sink.find(Name("b"));

                let f = toNumber(&*iters.next().unwrap().text());
                let s = toNumber(&*iters.next().unwrap().text());
                (f, s)
            }
            _ => (1.0, 1.0),
        };

        let s = U2client::reduceToText(&body, "基本信息");
        let size = U2client::matchRegex(&s, "大小:[' ']*([0-9.' ']+[TGMK]iB)");
        let number = toNumber(&*size);
        let GbSize = match size.chars().nth(size.len() - 3).unwrap() {
            'T' => number * 1024.0,
            'G' => number,
            'M' => number / 1024.0,
            _ => number / 1024.0 / 1024.0,
        };
        Ok(TorrentInfo {
            GbSize,
            uploadFX,
            downloadFX,
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

    fn matchRegex(src: &str, reg: &str) -> String {
        Regex::new(reg)
            .unwrap()
            .captures_iter(src)
            .next()
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .to_string()
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
