use serde::Deserialize;
use sysinfo::{Component, Disk, LoadAvg, Networks, ProcessorExt, System, SystemExt, User};

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

    pub RpcURL: String,
    pub RpcUsername: String,
    pub RpcPassword: String,
}

#[derive(Debug, Clone)]
pub struct HardwareStatus<'a> {
    pub cpu_useage: f32,
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub frequency: u64,
    pub vendor_id: &'a str,
    pub brand: &'a str,
    pub load_average: LoadAvg,
    pub temperature: &'a [Component],
    pub networks: &'a Networks,
    pub disks: &'a [Disk],
    pub users: &'a [User],
    pub uptime: (u64, u64, u64, u64),
    pub os_name: String,
    pub os_kernel_version: String,
    pub os_version: String,
    pub os_host_name: String,
}

impl<'T> HardwareStatus<'T> {
    pub fn new(sys: &'T System) -> HardwareStatus<'T> {
        let firstCore = sys.get_processors().get(0);
        let (frequency, vendor_id, brand) = if let Some(firstCore) = firstCore {
            (
                firstCore.get_frequency(),
                firstCore.get_vendor_id(),
                firstCore.get_brand(),
            )
        } else {
            (0, "<unknown>", "<unknown>")
        };
        HardwareStatus {
            cpu_useage: sys.get_global_processor_info().get_cpu_usage(),
            total_memory: sys.get_total_memory(),
            used_memory: sys.get_used_memory(),
            total_swap: sys.get_total_swap(),
            used_swap: sys.get_used_swap(),
            frequency,
            vendor_id,
            brand,
            load_average: sys.get_load_average(),
            temperature: sys.get_components(),
            networks: sys.get_networks(),
            disks: sys.get_disks(),
            users: sys.get_users(),
            uptime: {
                let mut uptime = sys.get_uptime();
                let days = uptime / 86400;
                uptime -= days * 86400;
                let hours = uptime / 3600;
                uptime -= hours * 3600;
                let minutes = uptime / 60;
                uptime -= minutes * 60;
                (days, hours, minutes, uptime)
            },
            os_name: sys.get_name().unwrap_or_else(|| "<unknown>".to_owned()),
            os_kernel_version: sys
                .get_kernel_version()
                .unwrap_or_else(|| "<unknown>".to_owned()),
            os_version: sys
                .get_os_version()
                .unwrap_or_else(|| "<unknown>".to_owned()),
            os_host_name: sys
                .get_host_name()
                .unwrap_or_else(|| "<unknown>".to_owned()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Status<'T> {
    pub hardware: HardwareStatus<'T>,
    pub local: crate::torrentLib::response::SessionStats,
    pub remote: UserInfo,
}
