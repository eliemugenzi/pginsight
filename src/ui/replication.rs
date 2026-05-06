use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::app::{App, Tab};
use crate::format;

pub fn draw(f: &mut Frame<'_>, app: &App, area: Rect) {
    let block = super::panel("Replication");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(ref rep) = app.replication else {
        f.render_widget(
            Paragraph::new(Span::styled("Loading…", Style::default().fg(Color::DarkGray))),
            inner,
        );
        return;
    };

    // Layout: summary | replicas | slots
    let has_slots = !rep.slots.is_empty();
    let constraints = if has_slots {
        vec![
            Constraint::Length(3),   // summary
            Constraint::Min(5),      // replicas
            Constraint::Length(4 + rep.slots.len() as u16 + 1), // slots
        ]
    } else {
        vec![
            Constraint::Length(3),
            Constraint::Min(5),
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    draw_summary(f, rep, chunks[0]);
    draw_replicas(f, app, rep, chunks[1]);
    if has_slots {
        draw_slots(f, rep, chunks[2]);
    }
}

fn draw_summary(f: &mut Frame<'_>, rep: &crate::db::replication::ReplicationInfo, area: Rect) {
    let (role_text, role_style) = if rep.is_in_recovery {
        (
            "Replica",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            "Primary",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    };

    let lsn = rep
        .current_wal_lsn
        .as_deref()
        .or(rep.receive_lsn.as_deref())
        .unwrap_or("-");

    let lsn_label = if rep.is_in_recovery {
        "Receive LSN"
    } else {
        "Current WAL LSN"
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Role            ", Style::default().fg(Color::DarkGray)),
            Span::styled(role_text, role_style),
            Span::raw("    "),
            Span::styled(lsn_label, Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::raw(lsn),
            Span::raw("    "),
            Span::styled(
                format!("{} replica(s)", rep.replicas.len()),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(lines), area);
}

fn draw_replicas(
    f: &mut Frame<'_>,
    app: &App,
    rep: &crate::db::replication::ReplicationInfo,
    area: Rect,
) {
    if rep.replicas.is_empty() {
        f.render_widget(
            Paragraph::new(Span::styled(
                if rep.is_in_recovery {
                    "Running as a replica — no downstream replicas."
                } else {
                    "No replicas connected."
                },
                Style::default().fg(Color::DarkGray),
            )),
            area,
        );
        return;
    }

    let header = Row::new(vec![
        "Application", "Addr", "State", "Sync", "Sent LSN", "Replay LSN", "Write Lag", "Flush Lag", "Replay Lag",
    ])
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    let rows: Vec<Row> = rep
        .replicas
        .iter()
        .map(|r| {
            let replay_lag_style = r
                .replay_lag_ms
                .map(|ms| {
                    if ms > 10_000.0 {
                        Style::default().fg(Color::Red)
                    } else if ms > 1_000.0 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Green)
                    }
                })
                .unwrap_or_default();

            let sync_style = match r.sync_state.as_deref() {
                Some("sync") | Some("quorum") => Style::default().fg(Color::Green),
                Some("async") => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::DarkGray),
            };

            Row::new(vec![
                Cell::from(r.application_name.as_deref().unwrap_or("-").to_string()),
                Cell::from(r.client_addr.as_deref().unwrap_or("-").to_string()),
                Cell::from(r.state.as_deref().unwrap_or("-").to_string()),
                Cell::from(r.sync_state.as_deref().unwrap_or("-").to_string()).style(sync_style),
                Cell::from(r.sent_lsn.as_deref().unwrap_or("-").to_string()),
                Cell::from(r.replay_lsn.as_deref().unwrap_or("-").to_string()),
                Cell::from(
                    r.write_lag_ms.map(format::ms).unwrap_or_else(|| "-".into()),
                ),
                Cell::from(
                    r.flush_lag_ms.map(format::ms).unwrap_or_else(|| "-".into()),
                ),
                Cell::from(
                    r.replay_lag_ms
                        .map(format::ms)
                        .unwrap_or_else(|| "-".into()),
                )
                .style(replay_lag_style),
            ])
        })
        .collect();

    let selected_idx = app.selected[Tab::Replication.index()];
    let table = Table::new(
        rows,
        [
            Constraint::Length(14), // Application
            Constraint::Length(16), // Addr
            Constraint::Length(12), // State
            Constraint::Length(8),  // Sync
            Constraint::Length(14), // Sent LSN
            Constraint::Length(14), // Replay LSN
            Constraint::Length(11), // Write Lag
            Constraint::Length(11), // Flush Lag
            Constraint::Length(11), // Replay Lag
        ],
    )
    .header(header)
    .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
    .highlight_symbol("▶ ");

    let mut state = TableState::default().with_selected(Some(selected_idx));
    f.render_stateful_widget(table, area, &mut state);
}

fn draw_slots(f: &mut Frame<'_>, rep: &crate::db::replication::ReplicationInfo, area: Rect) {
    let header = Row::new(vec!["Slot Name", "Type", "Plugin", "Active", "Retained WAL"])
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = rep
        .slots
        .iter()
        .map(|s| {
            let active_style = if s.active {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            let retained = s
                .retained_bytes
                .map(format::bytes)
                .unwrap_or_else(|| "-".into());
            let retained_style = s
                .retained_bytes
                .map(|b| {
                    if b > 10 * 1024 * 1024 * 1024 {
                        // > 10 GiB
                        Style::default().fg(Color::Red)
                    } else if b > 1024 * 1024 * 1024 {
                        // > 1 GiB
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    }
                })
                .unwrap_or_default();

            Row::new(vec![
                Cell::from(s.slot_name.clone()),
                Cell::from(s.slot_type.clone()),
                Cell::from(s.plugin.as_deref().unwrap_or("-").to_string()),
                Cell::from(if s.active { "yes" } else { "no" }).style(active_style),
                Cell::from(retained).style(retained_style),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(16),    // Slot Name
            Constraint::Length(12), // Type
            Constraint::Length(14), // Plugin
            Constraint::Length(8),  // Active
            Constraint::Length(14), // Retained WAL
        ],
    )
    .header(header);

    f.render_widget(table, area);
}
