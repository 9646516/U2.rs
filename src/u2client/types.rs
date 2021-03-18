use serde::Deserialize;
use sysinfo::System;

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub username: String,

    pub download: String,
    pub upload: String,
    pub shareRate: String,

    pub actualDownload: String,
    pub actualUpload: String,

    pub coin: String,

    pub downloadTime: String,
    pub uploadTime: String,
    pub timeRate: String,
}

#[derive(Debug, Clone)]
pub struct TorrentInfo {
    pub GbSize: f32,
    pub uploadFX: f32,
    pub downloadFX: f32,
    pub seeder: i32,
    pub leecher: i32,
    pub avgProgress: f32,
    pub Hash: String,
}

#[derive(Debug, Clone)]
pub struct RssInfo {
    pub title: String,
    pub url: String,
    pub cat: String,
    pub U2Info: TorrentInfo,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub cookie: String,
    pub passkey: String,
    pub workRoot: String,
    pub proxy: Option<String>,

    pub downloadFx: Option<f32>,

    pub RpcURL: String,
    pub RpcUsername: String,
    pub RpcPassword: String,

    pub LogRoot: String,

    pub maxSize: f32,
}

#[derive(Debug)]
pub struct Status {
    pub hardware: Option<System>,
    pub local: Option<crate::torrentLib::response::SessionStats>,
    pub remote: Option<UserInfo>,
    pub logDir: Option<String>,
}
