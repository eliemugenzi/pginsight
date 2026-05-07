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
    if app.statements.is_empty() {
        let block = super::panel("Queries (pg_stat_statements)");
        let inner = block.inner(area);
        f.render_widget(block, area);

        let (msg, color) = if app.last_refresh.is_none() {
            ("Loading…".to_string(), Color::DarkGray)
        } else if let Some(ref err) = app.statements_error {
            let hint = if err.contains("shared_preload_libraries") {
                "\n\nFix: add pg_stat_statements to shared_preload_libraries in postgresql.conf, \
                 then restart PostgreSQL.\n\n\
                 shared_preload_libraries = 'pg_stat_statements'\n\n\
                 After restart, run: CREATE EXTENSION IF NOT EXISTS pg_stat_statements;"
            } else if err.contains("not installed") || err.contains("does not exist") {
                "\n\nFix: CREATE EXTENSION pg_stat_statements;"
            } else if err.contains("UPDATE") {
                "\n\nFix: ALTER EXTENSION pg_stat_statements UPDATE;"
            } else {
                ""
            };
            (format!("{err}{hint}"), Color::Red)
        } else {
            ("No statements recorded yet — run some queries first.".to_string(), Color::DarkGray)
        };

        f.render_widget(
            Paragraph::new(msg)
                .style(Style::default().fg(color))
                .wrap(Wrap { trim: false }),
            inner,
        );
        return;
    }

    let title = format!(
        "Queries — top {} by total time (pg_stat_statements)",
        app.statements.len()
    );
    let block = super::panel(&title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(5)])
        .split(inner);

    draw_table(f, app, chunks[0]);
    draw_query_detail(f, app, chunks[1]);
}

fn draw_table(f: &mut Frame<'_>, app: &App, area: Rect) {
    let selected_idx = app.selected[Tab::Queries.index()];

    let header = Row::new(vec!["Calls", "Total", "Mean", "Rows", "Query"])
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .statements
        .iter()
        .map(|s| {
            let mean_style = if s.mean_exec_ms > 1_000.0 {
                Style::default().fg(Color::Red)
            } else if s.mean_exec_ms > 100.0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Green)
            };

            let query = s.query.replace('\n', " ");
            let query = if query.len() > 80 {
                format!("{}…", &query[..80])
            } else {
                query
            };

            Row::new(vec![
                Cell::from(format::number(s.calls)),
                Cell::from(format::ms(s.total_exec_ms)),
                Cell::from(format::ms(s.mean_exec_ms)).style(mean_style),
                Cell::from(format::number(s.rows)),
                Cell::from(query),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10), // Calls
            Constraint::Length(12), // Total
            Constraint::Length(10), // Mean
            Constraint::Length(12), // Rows
            Constraint::Min(20),    // Query
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

fn draw_query_detail(f: &mut Frame<'_>, app: &App, area: Rect) {
    let idx = app.selected[Tab::Queries.index()];
    let stmt = app.statements.get(idx);

    let meta = stmt
        .map(|s| {
            format!(
                "calls:{} · total:{} · mean:{} · rows:{}",
                format::number(s.calls),
                format::ms(s.total_exec_ms),
                format::ms(s.mean_exec_ms),
                format::number(s.rows),
            )
        })
        .unwrap_or_default();

    let mut lines = vec![Line::from(Span::styled(
        meta,
        Style::default().fg(Color::DarkGray),
    ))];

    match stmt.map(|s| s.query.trim()) {
        None | Some("") => {
            lines.push(Line::from(Span::styled(
                "(select a row above)",
                Style::default().fg(Color::DarkGray),
            )));
        }
        Some(q) => {
            lines.extend(crate::sql_format::highlight(q));
        }
    }

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}
