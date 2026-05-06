use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::app::{App, Tab};
use crate::format;

pub fn draw(f: &mut Frame<'_>, app: &App, area: Rect) {
    if app.stats_show_tables {
        draw_tables(f, app, area);
    } else {
        draw_databases(f, app, area);
    }
}

fn draw_databases(f: &mut Frame<'_>, app: &App, area: Rect) {
    let title = format!("Stats — Databases ({})  [s] switch to Tables", app.db_stats.len());
    let block = super::panel(&title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.db_stats.is_empty() {
        f.render_widget(
            Paragraph::new(Span::styled(
                "Loading…",
                Style::default().fg(Color::DarkGray),
            )),
            inner,
        );
        return;
    }

    let header = Row::new(vec!["Database", "Size", "Conns", "Cache Hit", "Commits", "Rollbacks"])
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .db_stats
        .iter()
        .map(|d| {
            let cache = d
                .cache_hit_ratio
                .map(|r| {
                    let style = if r < 0.90 {
                        Style::default().fg(Color::Red)
                    } else if r < 0.95 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Green)
                    };
                    (format!("{:.1}%", r * 100.0), style)
                })
                .unwrap_or_else(|| ("-".into(), Style::default().fg(Color::DarkGray)));

            let rollback_style = if d.xact_rollback > 0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            Row::new(vec![
                Cell::from(d.datname.clone()),
                Cell::from(format::bytes(d.size_bytes)),
                Cell::from(d.connections.to_string()),
                Cell::from(cache.0).style(cache.1),
                Cell::from(format::number(d.xact_commit)),
                Cell::from(format::number(d.xact_rollback)).style(rollback_style),
            ])
        })
        .collect();

    let selected_idx = app.selected[Tab::Stats.index()];
    let table = Table::new(
        rows,
        [
            Constraint::Min(16),    // Database
            Constraint::Length(12), // Size
            Constraint::Length(7),  // Conns
            Constraint::Length(10), // Cache Hit
            Constraint::Length(14), // Commits
            Constraint::Length(12), // Rollbacks
        ],
    )
    .header(header)
    .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
    .highlight_symbol("▶ ");

    let mut state = TableState::default().with_selected(Some(selected_idx));
    f.render_stateful_widget(table, inner, &mut state);
}

fn draw_tables(f: &mut Frame<'_>, app: &App, area: Rect) {
    let title = format!("Stats — Tables (top {})  [s] switch to Databases", app.table_stats.len());
    let block = super::panel(&title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.table_stats.is_empty() {
        f.render_widget(
            Paragraph::new(Span::styled(
                "No user tables found.",
                Style::default().fg(Color::DarkGray),
            )),
            inner,
        );
        return;
    }

    let header = Row::new(vec!["Table", "Size", "Live Rows", "Dead Rows", "Dead %", "Seq Scans", "Idx Scans"])
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .table_stats
        .iter()
        .map(|t| {
            let full_name = format!("{}.{}", t.schema, t.name);
            let total_tup = t.n_live_tup + t.n_dead_tup;
            let dead_pct = if total_tup > 0 {
                t.n_dead_tup as f64 / total_tup as f64
            } else {
                0.0
            };
            let dead_style = if dead_pct > 0.20 {
                Style::default().fg(Color::Red)
            } else if dead_pct > 0.05 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Green)
            };

            let total_scans = t.seq_scan + t.idx_scan;
            let seq_pct = if total_scans > 0 {
                t.seq_scan as f64 / total_scans as f64
            } else {
                0.0
            };
            let seq_style = if seq_pct > 0.5 && total_scans > 100 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(full_name),
                Cell::from(format::bytes(t.total_size_bytes)),
                Cell::from(format::number(t.n_live_tup)),
                Cell::from(format::number(t.n_dead_tup)).style(dead_style),
                Cell::from(format!("{:.1}%", dead_pct * 100.0)).style(dead_style),
                Cell::from(format::number(t.seq_scan)).style(seq_style),
                Cell::from(format::number(t.idx_scan)),
            ])
        })
        .collect();

    let selected_idx = app.selected[Tab::Stats.index()];
    let table = Table::new(
        rows,
        [
            Constraint::Min(20),    // Table
            Constraint::Length(12), // Size
            Constraint::Length(12), // Live Rows
            Constraint::Length(12), // Dead Rows
            Constraint::Length(8),  // Dead %
            Constraint::Length(11), // Seq Scans
            Constraint::Length(11), // Idx Scans
        ],
    )
    .header(header)
    .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
    .highlight_symbol("▶ ");

    let mut state = TableState::default().with_selected(Some(selected_idx));
    f.render_stateful_widget(table, inner, &mut state);
}

// Make Constraint available in this module without a top-level use
use ratatui::layout::Constraint;
