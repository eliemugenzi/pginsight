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
        let msg = if app.last_refresh.is_none() {
            "Loading…"
        } else {
            "pg_stat_statements not installed.\n\
             Run: CREATE EXTENSION pg_stat_statements;\n\
             Then restart (or reload) PostgreSQL and reconnect."
        };
        f.render_widget(
            Paragraph::new(msg)
                .style(Style::default().fg(Color::DarkGray))
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

    let query = stmt.map(|s| s.query.trim()).unwrap_or("").to_string();
    let query = if query.len() > 400 {
        format!("{}…", &query[..400])
    } else {
        query
    };

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

    let lines = vec![
        Line::from(Span::styled(meta, Style::default().fg(Color::DarkGray))),
        Line::from(if query.is_empty() {
            Span::styled("(select a row above)", Style::default().fg(Color::DarkGray))
        } else {
            Span::raw(query)
        }),
    ];
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}
