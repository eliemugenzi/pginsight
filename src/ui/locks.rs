use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

use crate::app::{App, Tab};
use crate::format;

pub fn draw(f: &mut Frame<'_>, app: &App, area: Rect) {
    let title = if app.locks.is_empty() {
        "Locks — no lock waits".to_string()
    } else {
        format!("Locks — {} blocked session(s)", app.locks.len())
    };
    let block = super::panel(&title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.locks.is_empty() {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "No lock waits — all sessions running freely.",
                Style::default().fg(Color::Green),
            ))),
            inner,
        );
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(6)])
        .split(inner);

    draw_table(f, app, chunks[0]);
    draw_detail(f, app, chunks[1]);
}

fn draw_table(f: &mut Frame<'_>, app: &App, area: Rect) {
    let selected_idx = app.selected[Tab::Locks.index()];

    let header = Row::new(vec![
        "Blocked PID", "Blocked User", "Blocking PID", "Blocking User", "Lock Mode", "Relation", "Waiting",
    ])
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .locks
        .iter()
        .map(|lw| {
            let wait = format::ms(lw.wait_ms);
            let wait_style = if lw.wait_ms > 30_000.0 {
                Style::default().fg(Color::Red)
            } else if lw.wait_ms > 5_000.0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            Row::new(vec![
                Cell::from(lw.blocked_pid.to_string())
                    .style(Style::default().fg(Color::Red)),
                Cell::from(lw.blocked_user.as_deref().unwrap_or("-").to_string()),
                Cell::from(lw.blocking_pid.to_string())
                    .style(Style::default().fg(Color::Yellow)),
                Cell::from(lw.blocking_user.as_deref().unwrap_or("-").to_string()),
                Cell::from(lw.lock_mode.as_deref().unwrap_or("-").to_string()),
                Cell::from(lw.relation.as_deref().unwrap_or("-").to_string()),
                Cell::from(wait).style(wait_style),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(13), // Blocked PID
            Constraint::Length(14), // Blocked User
            Constraint::Length(14), // Blocking PID
            Constraint::Length(14), // Blocking User
            Constraint::Length(22), // Lock Mode
            Constraint::Length(18), // Relation
            Constraint::Length(12), // Waiting
        ],
    )
    .header(header)
    .highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    let mut state = TableState::default().with_selected(Some(selected_idx));
    f.render_stateful_widget(table, area, &mut state);
}

fn draw_detail(f: &mut Frame<'_>, app: &App, area: Rect) {
    let idx = app.selected[Tab::Locks.index()];
    let Some(lw) = app.locks.get(idx) else {
        return;
    };

    let blocked_q = lw.blocked_query.as_deref().unwrap_or("(none)").trim();
    let blocking_q = lw.blocking_query.as_deref().unwrap_or("(none)").trim();

    let trunc = |s: &str, n: usize| -> String {
        if s.len() > n {
            format!("{}…", &s[..n])
        } else {
            s.to_string()
        }
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Blocked   ", Style::default().fg(Color::Red)),
            Span::raw(trunc(blocked_q, 120)),
        ]),
        Line::from(vec![
            Span::styled("Blocking  ", Style::default().fg(Color::Yellow)),
            Span::raw(trunc(blocking_q, 120)),
        ]),
    ];
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}
