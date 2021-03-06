use enum_iterator::IntoEnumIterator;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct RpcRequest {
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    arguments: Option<Args>,
}

impl RpcRequest {
    pub fn session_get() -> RpcRequest {
        RpcRequest {
            method: String::from("session-get"),
            arguments: None,
        }
    }

    pub fn session_stats() -> RpcRequest {
        RpcRequest {
            method: String::from("session-stats"),
            arguments: None,
        }
    }

    pub fn free_space(dir: String) -> RpcRequest {
        RpcRequest {
            method: String::from("free-space"),
            arguments: Some(Args::FreeSpaceArgs(FreeSpaceArgs { path: dir })),
        }
    }

    pub fn torrent_get(fields: Option<Vec<TorrentGetField>>, ids: Option<Vec<Id>>) -> RpcRequest {
        let string_fields = fields
            .unwrap_or_else(TorrentGetField::all)
            .iter()
            .map(|f| f.to_str())
            .collect();
        RpcRequest {
            method: String::from("torrent-get"),
            arguments: Some(Args::TorrentGetArgs(TorrentGetArgs {
                fields: Some(string_fields),
                ids,
            })),
        }
    }

    pub fn torrent_remove(ids: Vec<Id>, delete_local_data: bool) -> RpcRequest {
        RpcRequest {
            method: String::from("torrent-remove"),
            arguments: Some(Args::TorrentRemoveArgs(TorrentRemoveArgs {
                ids,
                delete_local_data,
            })),
        }
    }

    pub fn torrent_add(add: TorrentAddArgs) -> RpcRequest {
        RpcRequest {
            method: String::from("torrent-add"),
            arguments: Some(Args::TorrentAddArgs(add)),
        }
    }

    pub fn torrent_action(action: TorrentAction, ids: Vec<Id>) -> RpcRequest {
        RpcRequest {
            method: action.to_str(),
            arguments: Some(Args::TorrentActionArgs(TorrentActionArgs { ids })),
        }
    }
}

pub trait ArgumentFields {}

impl ArgumentFields for TorrentGetField {}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum Args {
    TorrentGetArgs(TorrentGetArgs),
    TorrentActionArgs(TorrentActionArgs),
    TorrentRemoveArgs(TorrentRemoveArgs),
    TorrentAddArgs(TorrentAddArgs),
    FreeSpaceArgs(FreeSpaceArgs),
}

#[derive(Serialize, Debug, Clone)]
pub struct TorrentGetArgs {
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ids: Option<Vec<Id>>,
}

impl Default for TorrentGetArgs {
    fn default() -> Self {
        let all_fields = TorrentGetField::into_enum_iter()
            .map(|it| it.to_str())
            .collect();
        TorrentGetArgs {
            fields: Some(all_fields),
            ids: None,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct FreeSpaceArgs {
    path: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct TorrentActionArgs {
    ids: Vec<Id>,
}

#[derive(Serialize, Debug, Clone)]
pub struct TorrentRemoveArgs {
    ids: Vec<Id>,
    #[serde(rename = "delete-local-data")]
    delete_local_data: bool,
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum Id {
    Id(i64),
    Hash(String),
}

#[derive(Serialize, Debug, Clone)]
pub struct TorrentAddArgs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "download-dir")]
    pub download_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metainfo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
}

impl Default for TorrentAddArgs {
    fn default() -> Self {
        TorrentAddArgs {
            cookies: None,
            download_dir: None,
            filename: None,
            metainfo: None,
            paused: None,
        }
    }
}

#[derive(Clone, IntoEnumIterator)]
pub enum TorrentGetField {
    Id,
    Addeddate,
    Name,
    HashString,
    Totalsize,
    Error,
    Errorstring,
    Eta,
    Isfinished,
    Isstalled,
    Leftuntildone,
    Metadatapercentcomplete,
    Peersconnected,
    Peersgettingfromus,
    Peerssendingtous,
    Percentdone,
    Queueposition,
    Ratedownload,
    Rateupload,
    Recheckprogress,
    Seedratiomode,
    Seedratiolimit,
    Sizewhendone,
    Status,
    Trackers,
    Downloaddir,
    Uploadedever,
    Uploadratio,
    Webseedssendingtous,
}

impl TorrentGetField {
    pub fn all() -> Vec<TorrentGetField> {
        TorrentGetField::into_enum_iter().collect()
    }
}

impl TorrentGetField {
    pub fn to_str(&self) -> String {
        match self {
            TorrentGetField::Id => "id",
            TorrentGetField::Addeddate => "addedDate",
            TorrentGetField::Name => "name",
            TorrentGetField::HashString => "hashString",
            TorrentGetField::Totalsize => "totalSize",
            TorrentGetField::Error => "error",
            TorrentGetField::Errorstring => "errorString",
            TorrentGetField::Eta => "eta",
            TorrentGetField::Isfinished => "isFinished",
            TorrentGetField::Isstalled => "isStalled",
            TorrentGetField::Leftuntildone => "leftUntilDone",
            TorrentGetField::Metadatapercentcomplete => "metadataPercentComplete",
            TorrentGetField::Peersconnected => "peersConnected",
            TorrentGetField::Peersgettingfromus => "peersGettingFromUs",
            TorrentGetField::Peerssendingtous => "peersSendingToUs",
            TorrentGetField::Percentdone => "percentDone",
            TorrentGetField::Queueposition => "queuePosition",
            TorrentGetField::Ratedownload => "rateDownload",
            TorrentGetField::Rateupload => "rateUpload",
            TorrentGetField::Recheckprogress => "recheckProgress",
            TorrentGetField::Seedratiomode => "seedRatioMode",
            TorrentGetField::Seedratiolimit => "seedRatioLimit",
            TorrentGetField::Sizewhendone => "sizeWhenDone",
            TorrentGetField::Status => "status",
            TorrentGetField::Trackers => "trackers",
            TorrentGetField::Downloaddir => "downloadDir",
            TorrentGetField::Uploadedever => "uploadedEver",
            TorrentGetField::Uploadratio => "uploadRatio",
            TorrentGetField::Webseedssendingtous => "webseedsSendingToUs",
        }
        .to_string()
    }
}

pub enum TorrentAction {
    Start,
    Stop,
    StartNow,
    Verify,
    Reannounce,
}

impl TorrentAction {
    pub fn to_str(&self) -> String {
        match self {
            TorrentAction::Start => "torrent-start",
            TorrentAction::Stop => "torrent-stop",
            TorrentAction::StartNow => "torrent-start-now",
            TorrentAction::Verify => "torrent-verify",
            TorrentAction::Reannounce => "torrent-reannounce",
        }
        .to_string()
    }
}
