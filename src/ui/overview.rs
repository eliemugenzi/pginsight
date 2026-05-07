use chrono::Utc as ChronoUtc;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Gauge, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::format;

pub fn draw(f: &mut Frame<'_>, app: &App, area: Rect) {
    let block = super::panel("Overview");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(ref ov) = app.overview else {
        f.render_widget(
            Paragraph::new("Connecting…").style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    };

    // Layout: info row | gauges | connection breakdown
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // server info
            Constraint::Length(4), // gauges
            Constraint::Min(0),    // connection breakdown + db stats
        ])
        .split(inner);

    draw_server_info(f, app, ov, chunks[0]);
    draw_gauges(f, ov, chunks[1]);
    draw_bottom(f, ov, chunks[2]);
}

fn draw_server_info(
    f: &mut Frame<'_>,
    app: &App,
    ov: &crate::db::overview::Overview,
    area: Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let started = ov
        .started_at
        .as_ref()
        .map(|t| format::datetime(t))
        .unwrap_or_else(|| "unknown".into());

    let role = if ov.is_in_recovery {
        Span::styled("Replica  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("Primary  ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    };

    let left = vec![
        Line::from(vec![
            label("Version   "),
            Span::raw(&ov.server_version),
        ]),
        Line::from(vec![
            label("Role      "),
            role,
        ]),
        Line::from(vec![
            label("Started   "),
            Span::raw(started),
        ]),
        Line::from(vec![
            label("Uptime    "),
            Span::raw(ov.started_at.map(|t| {
                let secs = (ChronoUtc::now().signed_duration_since(t)).num_seconds().max(0) as u64;
                let d = secs / 86400;
                let h = (secs % 86400) / 3600;
                let m = (secs % 3600) / 60;
                if d > 0 {
                    format!("{}d {}h {:02}m", d, h, m)
                } else if h > 0 {
                    format!("{}h {:02}m", h, m)
                } else {
                    format!("{}m", m)
                }
            }).unwrap_or_else(|| "-".into())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  ? ",
                Style::default().bg(Color::DarkGray).fg(Color::White),
            ),
            Span::raw(" help   "),
            Span::styled(
                " j/k ",
                Style::default().bg(Color::DarkGray).fg(Color::White),
            ),
            Span::raw(" navigate tables"),
        ]),
    ];

    let right = vec![
        Line::from(vec![
            label("Database  "),
            Span::raw(&ov.current_database),
            Span::styled(
                format!("  ({})", format::bytes(ov.db_size_bytes)),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            label("User      "),
            Span::raw(&ov.current_user),
        ]),
        Line::from(vec![
            label("Host      "),
            Span::raw(format!("{}:{}", app.pool.host(), app.pool.port())),
        ]),
        Line::from(vec![
            label("Commits   "),
            Span::raw(format::number(ov.xact_commit)),
        ]),
        Line::from(vec![
            label("Rollbacks "),
            Span::styled(
                format::number(ov.xact_rollback),
                if ov.xact_rollback > 0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                },
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(left), chunks[0]);
    f.render_widget(Paragraph::new(right), chunks[1]);
}

fn draw_gauges(f: &mut Frame<'_>, ov: &crate::db::overview::Overview, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(2)])
        .split(area);

    // Connections gauge
    let conn_ratio = if ov.max_connections > 0 {
        (ov.total_connections as f64 / ov.max_connections as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let conn_color = if conn_ratio > 0.85 {
        Color::Red
    } else if conn_ratio > 0.65 {
        Color::Yellow
    } else {
        Color::Cyan
    };
    let conn_gauge = Gauge::default()
        .gauge_style(Style::default().fg(conn_color))
        .ratio(conn_ratio)
        .label(format!(
            "Connections  {}/{} ({:.0}%)",
            ov.total_connections,
            ov.max_connections,
            conn_ratio * 100.0
        ));
    f.render_widget(conn_gauge, chunks[0]);

    // Cache hit ratio gauge
    let cache_ratio = ov.cache_hit_ratio.clamp(0.0, 1.0);
    let cache_color = if cache_ratio < 0.90 {
        Color::Red
    } else if cache_ratio < 0.95 {
        Color::Yellow
    } else {
        Color::Green
    };
    let cache_gauge = Gauge::default()
        .gauge_style(Style::default().fg(cache_color))
        .ratio(cache_ratio)
        .label(format!("Cache Hit    {:.2}%", cache_ratio * 100.0));
    f.render_widget(cache_gauge, chunks[1]);
}

fn draw_bottom(f: &mut Frame<'_>, ov: &crate::db::overview::Overview, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            label("Active      "),
            Span::styled(
                ov.active_connections.to_string(),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            label("Idle        "),
            Span::styled(
                ov.idle_connections.to_string(),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("    "),
            label("Idle in Tx  "),
            Span::styled(
                ov.idle_in_tx_connections.to_string(),
                if ov.idle_in_tx_connections > 0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}

fn label(s: &'static str) -> Span<'static> {
    Span::styled(s, Style::default().fg(Color::DarkGray))
}
