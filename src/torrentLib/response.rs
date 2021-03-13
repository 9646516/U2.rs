use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RpcResponse<T> {
    pub arguments: T,
    pub result: String,
}

impl<T> RpcResponse<T> {
    pub fn is_ok(&self) -> bool {
        self.result == "success"
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct SessionGet {
    #[serde(rename = "blocklist-enabled")]
    pub blocklist_enabled: bool,
    #[serde(rename = "download-dir")]
    pub download_dir: String,
    pub encryption: String,
    #[serde(rename = "rpc-version")]
    pub rpc_version: i32,
    #[serde(rename = "rpc-version-minimum")]
    pub rpc_version_minimum: i32,
    pub version: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Torrent {
    #[serde(rename = "addedDate")]
    pub added_date: Option<i64>,
    #[serde(rename = "downloadDir")]
    pub download_dir: Option<String>,
    pub error: Option<i64>,
    #[serde(rename = "errorString")]
    pub error_string: Option<String>,
    pub eta: Option<i64>,
    pub id: Option<i64>,
    #[serde(rename = "isFinished")]
    pub is_finished: Option<bool>,
    #[serde(rename = "isStalled")]
    pub is_stalled: Option<bool>,
    #[serde(rename = "leftUntilDone")]
    pub left_until_done: Option<i64>,
    #[serde(rename = "metadataPercentComplete")]
    pub metadata_percent_complete: Option<f32>,
    pub name: Option<String>,
    #[serde(rename = "hashString")]
    pub hash_string: Option<String>,
    #[serde(rename = "peersConnected")]
    pub peers_connected: Option<i64>,
    #[serde(rename = "peersGettingFromUs")]
    pub peers_getting_from_us: Option<i64>,
    #[serde(rename = "peersSendingToUs")]
    pub peers_sending_to_us: Option<i64>,
    #[serde(rename = "percentDone")]
    pub percent_done: Option<f32>,
    #[serde(rename = "rateDownload")]
    pub rate_download: Option<i64>,
    #[serde(rename = "rateUpload")]
    pub rate_upload: Option<i64>,
    #[serde(rename = "recheckProgress")]
    pub recheck_progress: Option<f32>,
    #[serde(rename = "seedRatioLimit")]
    pub seed_ratio_limit: Option<f32>,
    #[serde(rename = "sizeWhenDone")]
    pub size_when_done: Option<i64>,
    pub status: Option<i64>,
    #[serde(rename = "totalSize")]
    pub total_size: Option<i64>,
    pub trackers: Option<Vec<Trackers>>,
    #[serde(rename = "uploadRatio")]
    pub upload_ratio: Option<f32>,
    #[serde(rename = "uploadedEver")]
    pub uploaded_ever: Option<i64>,
}

#[derive(Deserialize, Debug)]
pub struct Torrents<T> {
    pub torrents: Vec<T>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Trackers {
    pub id: i32,
    pub announce: String,
}

#[derive(Deserialize, Debug)]
pub struct Nothing {}

#[derive(Deserialize, Debug)]
pub struct TorrentAdded {
    #[serde(rename = "torrent-added")]
    pub torrent_added: Option<Torrent>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FreeSpace {
    pub path: Option<String>,
    #[serde(rename = "size-bytes")]
    pub size_bytes: Option<u64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SessionStats {
    pub activeTorrentCount: u64,
    pub downloadSpeed: u64,
    pub pausedTorrentCount: u64,
    pub torrentCount: u64,
    pub uploadSpeed: u64,
    #[serde(rename = "cumulative-stats")]
    pub cumulative_stats: Stats,
    #[serde(rename = "current-stats")]
    pub current_stats: Stats,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Stats {
    pub uploadedBytes: u64,
    pub downloadedBytes: u64,
    pub filesAdded: u64,
    pub sessionCount: u64,
    pub secondsActive: u64,
}
