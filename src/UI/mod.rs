use tui::{
    backend::Backend,
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
};

pub fn draw<B: Backend>(f: &mut Frame<B>) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .direction(Direction::Vertical)
        .split(f.size());

    draw2(f, chunks[0]);
}

fn draw2<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let items: Vec<Row> = (0..4)
        .map(|c| {
            let cells = vec![
                Cell::from(Span::raw(format!("{:?} ", c))),
                Cell::from(Span::styled("style", Style::default().fg(Color::Yellow))),
            ];
            Row::new(cells)
        })
        .collect();
    let table = Table::new(items)
        .block(Block::default().title("title").borders(Borders::ALL))
        .widths(&[Constraint::Percentage(30), Constraint::Percentage(30)]);
    f.render_widget(table, area);
}
