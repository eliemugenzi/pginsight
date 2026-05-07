mod activity;
mod help;
mod locks;
mod overview;
mod queries;
mod replication;
mod stats;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, Tab};

pub fn draw(f: &mut Frame<'_>, app: &App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Length(3), // tab bar
            Constraint::Min(0),    // tab body
            Constraint::Length(1), // status bar
        ])
        .split(area);

    draw_header(f, app, chunks[0]);
    draw_tabs(f, app, chunks[1]);
    draw_body(f, app, chunks[2]);
    draw_status(f, app, chunks[3]);

    if app.show_help {
        help::draw(f, area);
    }
}

fn draw_header(f: &mut Frame<'_>, app: &App, area: Rect) {
    let role = if app
        .replication
        .as_ref()
        .map(|r| r.is_in_recovery)
        .unwrap_or(false)
    {
        " [REPLICA]"
    } else {
        ""
    };

    let connection = format!(
        "{user}@{host}:{port}/{db}{role}",
        user = app.pool.user(),
        host = app.pool.host(),
        port = app.pool.port(),
        db = app.pool.dbname(),
    );

    let version = app
        .overview
        .as_ref()
        .map(|o| format!("  PostgreSQL {}", o.server_version))
        .unwrap_or_default();

    let line = Line::from(vec![
        Span::styled(
            "pginsight",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(connection, Style::default().fg(Color::Gray)),
        Span::styled(version, Style::default().fg(Color::DarkGray)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(Paragraph::new(line).block(block), area);
}

fn draw_tabs(f: &mut Frame<'_>, app: &App, area: Rect) {
    let titles: Vec<Line> = Tab::ALL
        .iter()
        .enumerate()
        .map(|(i, t)| {
            Line::from(vec![
                Span::styled(
                    format!(" {} ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(t.title()),
                Span::raw(" "),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(app.tab.index())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider("│");
    f.render_widget(tabs, area);
}

fn draw_body(f: &mut Frame<'_>, app: &App, area: Rect) {
    match app.tab {
        Tab::Overview => overview::draw(f, app, area),
        Tab::Activity => activity::draw(f, app, area),
        Tab::Queries => queries::draw(f, app, area),
        Tab::Stats => stats::draw(f, app, area),
        Tab::Locks => locks::draw(f, app, area),
        Tab::Replication => replication::draw(f, app, area),
    }
}

fn draw_status(f: &mut Frame<'_>, app: &App, area: Rect) {
    let line = if let Some(err) = &app.error {
        let msg = if err.len() > 80 {
            format!("{}…", &err[..80])
        } else {
            err.clone()
        };
        Line::from(vec![
            Span::styled(
                " ERROR ",
                Style::default()
                    .bg(Color::Red)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(msg, Style::default().fg(Color::Red)),
            Span::raw("  "),
            Span::styled(
                " Esc ",
                Style::default().bg(Color::DarkGray).fg(Color::White),
            ),
            Span::raw(" dismiss"),
        ])
    } else {
        let mode = if app.paused {
            Span::styled(
                " PAUSED ",
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                " live ",
                Style::default().bg(Color::DarkGray).fg(Color::White),
            )
        };

        let since = app
            .last_refresh
            .map(|t| format!(" {}s ago", t.elapsed().as_secs()))
            .unwrap_or_else(|| " …".into());

        Line::from(vec![
            Span::styled(" q ", Style::default().bg(Color::DarkGray).fg(Color::White)),
            Span::raw(" quit  "),
            Span::styled(" ? ", Style::default().bg(Color::DarkGray).fg(Color::White)),
            Span::raw(" help  "),
            Span::styled(" r ", Style::default().bg(Color::DarkGray).fg(Color::White)),
            Span::raw(" refresh  "),
            Span::styled(" p ", Style::default().bg(Color::DarkGray).fg(Color::White)),
            Span::raw("  "),
            mode,
            Span::styled(since, Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(" · every {}s", app.refresh_every.as_secs()),
                Style::default().fg(Color::DarkGray),
            ),
        ])
    };
    f.render_widget(Paragraph::new(line), area);
}

/// Standard rounded panel block used by the tab views.
pub(crate) fn panel(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(format!(" {title} "))
        .border_style(Style::default().fg(Color::DarkGray))
}
