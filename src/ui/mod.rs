use std::fs::File;
use std::io::BufReader;
use std::option::Option::Some;

use rev_lines::RevLines;
use sysinfo::{ComponentExt, DiskExt, NetworkExt, NetworksExt, ProcessorExt, System, SystemExt};
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

pub struct TabsState {
    pub index: usize,
}

const TITLE: &[&str] = &["Status", "BT", "LOG"];

impl TabsState {
    pub fn new() -> TabsState {
        TabsState { index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % 3;
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = 2;
        }
    }
}

impl Default for TabsState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn draw<B: Backend>(f: &mut Frame<B>, x: Status, mask: u8, idx: usize) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
        .direction(Direction::Vertical)
        .split(f.size());

    let items: Vec<Vec<Cell>> = vec![TITLE
        .iter()
        .map(|x| {
            if x == TITLE.get(idx).unwrap() {
                Cell::from(Span::styled(
                    x.to_owned(),
                    Style::default().fg(Color::Yellow),
                ))
            } else {
                Cell::from(Span::raw(x.to_owned()))
            }
        })
        .collect()];

    let items: Vec<Row> = items.into_iter().map(Row::new).collect();
    let table = Table::new(items)
        .block(Block::default().title("Tab").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
        ]);
    f.render_widget(table, chunks[0]);

    let area = chunks[1];
    match idx {
        0 => {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(20), Constraint::Percentage(40)].as_ref())
                .direction(Direction::Vertical)
                .split(area);

            drawRemoteInfo(f, chunks[0], &x.remote, (mask >> 1) & 1);
            drawHardwareInfo(f, chunks[1], &x.hardware);
        }
        1 => {
            drawLocalInfo(f, area, &x.local, mask & 1);
        }
        2 => {
            drawLog(f, area, &x.logDir);
        }
        _ => {}
    }
}

fn drawLog<B: Backend>(f: &mut Frame<B>, area: Rect, log: &Option<String>) {
    let F = |x: &Option<String>| -> crate::Result<Vec<Row>> {
        let x = x.as_ref().ok_or("")?;
        let file = File::open(x)?;
        let rev_lines = RevLines::new(BufReader::new(file))?;
        Ok(rev_lines
            .take(25)
            .map(|x| Row::new(vec![Cell::from(Span::raw(x))]))
            .collect::<Vec<Row>>())
    };
    let items =
        F(log).unwrap_or_else(|_| vec![Row::new(vec![Cell::from(Span::raw("failed to get log"))])]);
    let table = Table::new(items)
        .block(Block::default().title("Logs").borders(Borders::ALL))
        .widths(&[Constraint::Percentage(100)]);
    f.render_widget(table, area);
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

fn getUpdatedInfoCell(mask: u8) -> Cell<'static> {
    if mask != 0 {
        Cell::from(Span::styled(
            "Updated".to_string(),
            Style::default().fg(Color::Green),
        ))
    } else {
        Cell::from(Span::styled(
            "Outdated".to_string(),
            Style::default().fg(Color::Red),
        ))
    }
}

fn drawRemoteInfo<B: Backend>(f: &mut Frame<B>, area: Rect, x: &Option<UserInfo>, mask: u8) {
    let items: Vec<Vec<Cell>> = match x {
        Some(x) => {
            vec![
                vec![
                    Cell::from(Span::raw(format!("Welcome {}", x.username))),
                    Cell::from(Span::styled(
                        format!("coin {}", x.coin),
                        Style::default().fg(Color::Yellow),
                    )),
                    getUpdatedInfoCell(mask),
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
            ]
        }
        None => {
            vec![vec![Cell::from(Span::raw("loading".to_string()))]]
        }
    };

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

fn drawHardwareInfo<B: Backend>(f: &mut Frame<B>, area: Rect, sys: &Option<System>) {
    let items: Vec<Vec<Cell>> = match sys {
        Some(sys) => {
            let firstCore = sys.get_processors().get(0);
            let brand = if let Some(firstCore) = firstCore {
                firstCore.get_brand()
            } else {
                "Unknown CPU"
            };
            let (days, hours, minutes, secs) = splitTime(sys.get_uptime());

            let mut items = vec![
                vec![
                    Cell::from(Span::raw(brand.to_string())),
                    Cell::from(Span::raw(format!(
                        "{} days {} hours {} mins {}",
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
            ];

            let mut s = sys
                .get_disks()
                .iter()
                .map(|i| {
                    (
                        i.get_mount_point().to_str().unwrap().to_string(),
                        vec![
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
                        ],
                    )
                })
                .collect::<Vec<(String, Vec<Cell>)>>();

            macro_rules! gao {
                ($arg:tt) => {
                    ($arg).sort_by_key(|x| x.0.to_owned());
                    let s: Vec<Vec<Cell>> = ($arg).into_iter().map(|x| x.1).collect();
                    for i in s.into_iter() {
                        items.push(i);
                    }
                };
            }
            gao!(s);

            let mut s = sys
                .get_networks()
                .iter()
                .map(|(a, b)| {
                    let mut a = a.clone();
                    a.truncate(20);
                    (
                        a.to_owned(),
                        vec![
                            Cell::from(Span::raw(a)),
                            Cell::from(Span::raw(format!(
                                "Download {:.3}GB",
                                b.get_total_received() as f32 / 1e9
                            ))),
                            Cell::from(Span::raw(format!(
                                "Upload {:.3}GB",
                                b.get_total_transmitted() as f32 / 1e9
                            ))),
                        ],
                    )
                })
                .collect::<Vec<(String, Vec<Cell>)>>();
            gao!(s);

            let mut s = sys
                .get_components()
                .iter()
                .map(|x| {
                    (
                        x.get_label().to_string(),
                        vec![
                            Cell::from(Span::raw(x.get_label().to_string())),
                            Cell::from(Span::raw(format!("Temp {:.3}", x.get_temperature()))),
                            Cell::from(Span::raw(format!("Max {:.3}", x.get_max()))),
                        ],
                    )
                })
                .collect::<Vec<(String, Vec<Cell>)>>();
            gao!(s);

            items
        }
        None => {
            vec![vec![Cell::from(Span::raw("loading".to_string()))]]
        }
    };

    let items: Vec<Row> = items.into_iter().map(Row::new).collect();
    let table = Table::new(items)
        .block(Block::default().title("Hardware").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ]);
    f.render_widget(table, area);
}

fn drawLocalInfo<B: Backend>(f: &mut Frame<B>, area: Rect, x: &Option<SessionStats>, mask: u8) {
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        )
        .direction(Direction::Vertical)
        .split(area);

    let items: Vec<Vec<Cell>> = match x {
        Some(x) => {
            vec![
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
                    Cell::from(Span::raw(format!(
                        "uploadSpeed {:.3} MB/s",
                        x.uploadSpeed as f32 / 1e6
                    ))),
                    Cell::from(Span::raw(format!(
                        "downloadSpeed {:.3} MB/s",
                        x.downloadSpeed as f32 / 1e6
                    ))),
                    getUpdatedInfoCell(mask),
                ],
            ]
        }
        None => {
            vec![vec![Cell::from(Span::raw("loading".to_string()))]]
        }
    };
    let items: Vec<Row> = items.into_iter().map(Row::new).collect();
    let table = Table::new(items)
        .block(Block::default().title("BT").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ]);
    f.render_widget(table, chunks[0]);
    if let Some(x) = x {
        drawStats(f, chunks[1], &x.current_stats, "current");
        drawStats(f, chunks[2], &x.cumulative_stats, "cumulative");
    }
}

fn drawStats<B: Backend>(f: &mut Frame<B>, area: Rect, x: &Stats, head: &str) {
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
                "Active {} days {} hours {} mins {}",
                days, hours, minutes, secs
            ))),
        ],
    ];
    let items: Vec<Row> = items.into_iter().map(Row::new).collect();
    let table = Table::new(items)
        .block(Block::default().title(head).borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ]);
    f.render_widget(table, area);
}
