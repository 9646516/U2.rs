use sysinfo::{DiskExt, NetworkExt, NetworksExt, ProcessorExt, System, SystemExt};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::torrentLib::response::{SessionStats, Stats};
use crate::u2client::types::{Status, UserInfo};

pub fn draw<B: Backend>(f: &mut Frame<B>, x: Status) {
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(40),
                Constraint::Percentage(40),
            ]
            .as_ref(),
        )
        .direction(Direction::Vertical)
        .split(f.size());

    drawRemoteInfo(f, chunks[0], &x.remote);
    drawHardwareInfo(f, chunks[1], &x.hardware);
    drawLocalInfo(f, chunks[2], &x.local);
}

fn splitTime(mut time: u64) -> (u64, u64, u64, u64) {
    let days = time / 86400;
    time -= days * 86400;
    let hours = time / 3600;
    time -= hours * 3600;
    let minutes = time / 60;
    let secs = time - minutes * 60;
    (days, hours, minutes, secs)
}

fn drawRemoteInfo<B: Backend>(f: &mut Frame<B>, area: Rect, x: &UserInfo) {
    let items: Vec<Vec<Cell>> = vec![
        vec![
            Cell::from(Span::raw(format!("Welcome {}", x.username))),
            Cell::from(Span::styled(
                format!("coin {}", x.coin),
                Style::default().fg(Color::Yellow),
            )),
        ],
        vec![
            Cell::from(Span::raw(format!("Download {}", x.download))),
            Cell::from(Span::raw(format!("Upload {}", x.upload))),
            Cell::from(Span::styled(
                format!("shareRate {}", x.shareRate),
                Style::default().fg(Color::Yellow),
            )),
        ],
        vec![
            Cell::from(Span::raw(format!("Actual Download {}", x.actualDownload))),
            Cell::from(Span::raw(format!("Actual Upload {}", x.actualUpload))),
        ],
        vec![
            Cell::from(Span::raw(format!("Upload Time {}", x.uploadTime))),
            Cell::from(Span::raw(format!("Download Time {}", x.downloadTime))),
            Cell::from(Span::styled(
                format!("timeRate {}", x.timeRate),
                Style::default().fg(Color::Yellow),
            )),
        ],
    ];

    let items: Vec<Row> = items.into_iter().map(Row::new).collect();
    let table = Table::new(items)
        .block(Block::default().title("User").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ]);
    f.render_widget(table, area);
}

fn drawHardwareInfo<B: Backend>(f: &mut Frame<B>, area: Rect, sys: &System) {
    let firstCore = sys.get_processors().get(0);
    let brand = if let Some(firstCore) = firstCore {
        firstCore.get_brand()
    } else {
        "Unknown CPU"
    };
    let (days, hours, minutes, secs) = splitTime(sys.get_uptime());

    let mut items: Vec<Vec<Cell>> = vec![
        vec![
            Cell::from(Span::raw(brand.to_string())),
            Cell::from(Span::raw(format!(
                "{} days {} hours {} mins {} s",
                days, hours, minutes, secs
            ))),
        ],
        vec![
            Cell::from(Span::raw(format!(
                "Memory {:.3}GB/{:.3}GB",
                sys.get_used_memory() as f32 / 1e6,
                sys.get_total_memory() as f32 / 1e6
            ))),
            Cell::from(Span::raw(format!(
                "Swap {:.3}GB/{:.3}GB",
                sys.get_used_swap() as f32 / 1e6,
                sys.get_total_swap() as f32 / 1e6
            ))),
        ],
        vec![
            Cell::from(Span::raw(format!(
                "{} {}",
                sys.get_name().unwrap_or_else(|| "<unknown>".to_owned()),
                sys.get_kernel_version()
                    .unwrap_or_else(|| "<unknown>".to_owned())
            ))),
            Cell::from(Span::raw(format!(
                "Version {}",
                sys.get_os_version()
                    .unwrap_or_else(|| "<unknown>".to_owned())
            ))),
            Cell::from(Span::raw(
                sys.get_host_name()
                    .unwrap_or_else(|| "<unknown>".to_string()),
            )),
        ],
        vec![
            Cell::from(Span::raw(format!(
                "1 min load_average {}%",
                sys.get_load_average().one
            ))),
            Cell::from(Span::raw(format!(
                "5 min load_average {}%",
                sys.get_load_average().five
            ))),
            Cell::from(Span::raw(format!(
                "15 min load_average {}%",
                sys.get_load_average().fifteen
            ))),
        ],
    ];
    sys.get_disks().iter().for_each(|i| {
        items.push(vec![
            Cell::from(Span::raw(format!(
                "{:?} {}",
                i.get_type(),
                i.get_mount_point().to_str().unwrap()
            ))),
            Cell::from(Span::styled(
                format!("Free {:.3}GB", i.get_available_space() as f32 / 1e9),
                Style::default().fg(Color::Yellow),
            )),
            Cell::from(Span::raw(format!(
                "Total {:.3}GB",
                i.get_total_space() as f32 / 1e9
            ))),
        ])
    });

    sys.get_networks().iter().for_each(|(a, b)| {
        let mut a = a.clone();
        a.truncate(20);
        items.push(vec![
            Cell::from(Span::raw(a)),
            Cell::from(Span::raw(format!(
                "Download {:.3}GB",
                b.get_total_received() as f32 / 1e9
            ))),
            Cell::from(Span::raw(format!(
                "Upload {:.3}GB",
                b.get_total_transmitted() as f32 / 1e9
            ))),
        ])
    });

    let items: Vec<Row> = items.into_iter().map(Row::new).collect();
    let table = Table::new(items)
        .block(Block::default().title("Local").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ]);
    f.render_widget(table, area);
}

fn drawLocalInfo<B: Backend>(f: &mut Frame<B>, area: Rect, x: &SessionStats) {
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(35),
                Constraint::Percentage(35),
            ]
            .as_ref(),
        )
        .direction(Direction::Vertical)
        .split(area);

    let items: Vec<Vec<Cell>> = vec![
        vec![
            Cell::from(Span::raw(format!(
                "Active Torrents {}",
                x.activeTorrentCount
            ))),
            Cell::from(Span::raw(format!(
                "Paused Torrents {}",
                x.pausedTorrentCount
            ))),
            Cell::from(Span::raw(format!("Torrents {}", x.torrentCount))),
        ],
        vec![
            Cell::from(Span::raw(format!("uploadSpeed {}", x.uploadSpeed))),
            Cell::from(Span::raw(format!("downloadSpeed {}", x.downloadSpeed))),
        ],
    ];
    let items: Vec<Row> = items.into_iter().map(Row::new).collect();
    let table = Table::new(items)
        .block(Block::default().title("BT").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ]);
    f.render_widget(table, chunks[0]);
    drawStats(f, chunks[1], &x.current_stats, "current");
    drawStats(f, chunks[2], &x.cumulative_stats, "cumulative");
}

fn drawStats<B: Backend>(f: &mut Frame<B>, area: Rect, x: &Stats, title: &str) {
    let (days, hours, minutes, secs) = splitTime(x.secondsActive);
    let items: Vec<Vec<Cell>> = vec![
        vec![
            Cell::from(Span::raw(format!(
                "Uploaded {:.3}GB",
                x.uploadedBytes as f32 / 1e9
            ))),
            Cell::from(Span::raw(format!(
                "Downloaded {:.3}GB",
                x.downloadedBytes as f32 / 1e9
            ))),
        ],
        vec![
            Cell::from(Span::raw(format!("Files Added {}", x.filesAdded))),
            Cell::from(Span::raw(format!("Sessions {}", x.sessionCount))),
            Cell::from(Span::raw(format!(
                "Active {} days {} hours {} mins {} s",
                days, hours, minutes, secs
            ))),
        ],
    ];
    let items: Vec<Row> = items.into_iter().map(Row::new).collect();
    let table = Table::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ]);
    f.render_widget(table, area);
}
