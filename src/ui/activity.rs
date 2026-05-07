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
    let filtered: Vec<&crate::db::activity::Session> = app
        .sessions
        .iter()
        .filter(|s| {
            app.activity_filter
                .matches(s.state.as_deref(), s.wait_event.as_deref())
        })
        .collect();

    let filter_label = app.activity_filter.label();
    let title = if app.sessions.is_empty() {
        "Activity — no client sessions  [f] filter".to_string()
    } else {
        format!(
            "Activity — {} of {} sessions  [f] filter: {}",
            filtered.len(),
            app.sessions.len(),
            filter_label,
        )
    };

    let block = super::panel(&title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.sessions.is_empty() {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "No client backends connected.",
                Style::default().fg(Color::DarkGray),
            ))),
            inner,
        );
        return;
    }

    if filtered.is_empty() {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("No sessions with state: {filter_label}"),
                Style::default().fg(Color::DarkGray),
            ))),
            inner,
        );
        return;
    }

    // Split: table on top, query detail on bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(4)])
        .split(inner);

    draw_table(f, app, &filtered, chunks[0]);
    draw_query_detail(f, app, &filtered, chunks[1]);
}

fn draw_table(
    f: &mut Frame<'_>,
    app: &App,
    sessions: &[&crate::db::activity::Session],
    area: Rect,
) {
    let selected_idx = app.selected[Tab::Activity.index()];

    let header = Row::new(vec!["PID", "User", "Database", "State", "Wait", "Duration", "Query"])
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = sessions
        .iter()
        .map(|s| {
            let state = s.state.as_deref().unwrap_or("");
            let state_style = state_style(state);

            let wait = match (&s.wait_event_type, &s.wait_event) {
                (Some(t), Some(e)) => format!("{}/{}", t, e),
                (Some(t), None) => t.clone(),
                _ => String::new(),
            };
            let wait_style = if wait.is_empty() {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Yellow)
            };

            let dur = if s.duration_ms > 0.1 {
                format::ms(s.duration_ms)
            } else {
                String::new()
            };
            let dur_style = if s.duration_ms > 5_000.0 {
                Style::default().fg(Color::Red)
            } else if s.duration_ms > 1_000.0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let query = s.query.as_deref().unwrap_or("").replace('\n', " ");
            let query = if query.len() > 80 {
                format!("{}…", &query[..80])
            } else {
                query
            };

            Row::new(vec![
                Cell::from(s.pid.to_string()),
                Cell::from(s.usename.as_deref().unwrap_or("-").to_string()),
                Cell::from(s.datname.as_deref().unwrap_or("-").to_string()),
                Cell::from(state.to_string()).style(state_style),
                Cell::from(wait).style(wait_style),
                Cell::from(dur).style(dur_style),
                Cell::from(query),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),   // PID
            Constraint::Length(12),  // User
            Constraint::Length(14),  // Database
            Constraint::Length(22),  // State
            Constraint::Length(18),  // Wait
            Constraint::Length(10),  // Duration
            Constraint::Min(20),     // Query
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

fn draw_query_detail(
    f: &mut Frame<'_>,
    app: &App,
    sessions: &[&crate::db::activity::Session],
    area: Rect,
) {
    let idx = app.selected[Tab::Activity.index()];
    let session = sessions.get(idx).copied();

    let app_name = session
        .and_then(|s| s.application_name.as_deref())
        .filter(|s| !s.is_empty())
        .map(|s| format!("  app:{s}"))
        .unwrap_or_default();
    let client = session
        .and_then(|s| s.client_addr.as_deref())
        .map(|a| format!("  from:{a}"))
        .unwrap_or_default();
    let pid_str = session.map(|s| format!("pid:{}", s.pid)).unwrap_or_default();

    let mut lines = vec![Line::from(Span::styled(
        format!("{pid_str}{app_name}{client}"),
        Style::default().fg(Color::DarkGray),
    ))];

    let query = session.and_then(|s| s.query.as_deref()).unwrap_or("").trim();
    if query.is_empty() {
        lines.push(Line::from(Span::styled(
            "(no query)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        lines.extend(crate::sql_format::highlight(query));
    }

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}

fn state_style(state: &str) -> Style {
    match state {
        "active" => Style::default().fg(Color::Green),
        "idle" => Style::default().fg(Color::DarkGray),
        s if s.starts_with("idle in transaction") => Style::default().fg(Color::Yellow),
        "disabled" => Style::default().fg(Color::DarkGray),
        _ => Style::default(),
    }
}
