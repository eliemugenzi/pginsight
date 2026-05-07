//! Help overlay.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame<'_>, area: Rect) {
    let area = centered(70, 80, area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Help — pginsight keybindings (any key to dismiss) ")
        .border_style(Style::default().fg(Color::Cyan));

    let lines = vec![
        section("Navigation"),
        kb("Tab / Shift+Tab",      "next / previous tab"),
        kb("1 .. 6",               "jump directly to a tab"),
        kb("↑ / ↓  or  k / j",      "move selection"),
        kb("PgUp / PgDn",          "page through rows"),
        kb("g / G",                "jump to top / bottom"),
        Line::from(""),
        section("Refresh"),
        kb("r",                    "refresh now"),
        kb("p",                    "pause / resume auto-refresh"),
        Line::from(""),
        section("Activity tab"),
        kb("x",                    "pg_cancel_backend(pid) on selected session"),
        kb("Shift+K",              "pg_terminate_backend(pid) on selected session"),
        kb("+ / -",                "increase / decrease 'min duration' filter"),
        Line::from(""),
        section("Queries tab"),
        kb("e",                    "EXPLAIN the selected statement"),
        kb("Shift+R",              "pg_stat_statements_reset()"),
        Line::from(""),
        section("Stats tab"),
        kb("s",                    "toggle Databases ↔ Tables view"),
        Line::from(""),
        section("Output"),
        kb("o",                    "export current tab to JSON in cwd"),
        Line::from(""),
        section("Misc"),
        kb("? / F1",               "show / hide this help"),
        kb("Esc",                  "dismiss overlay or error"),
        kb("q  or  Ctrl+C",        "quit"),
    ];

    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn section(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!(" {title}"),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    ))
}

fn kb(key: &'static str, desc: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("{:<18}", key),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::raw(desc),
    ])
}

fn centered(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(v[1])[1]
}
