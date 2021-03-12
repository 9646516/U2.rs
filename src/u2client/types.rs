#[derive(Debug, Clone)]
pub struct UserInfo {
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
    pub avgProgress: String,
}

#[derive(Debug, Clone)]
pub struct RssInfo {
    pub title: String,
    pub url: String,
    pub cat: String,
    pub U2Info: TorrentInfo,
}