use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use std::time::{Duration, Instant};

use crate::db;
use crate::db::Pool;
use crate::tui::Tui;
use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Activity,
    Queries,
    Stats,
    Locks,
    Replication,
}

impl Tab {
    pub const ALL: [Tab; 6] = [
        Tab::Overview,
        Tab::Activity,
        Tab::Queries,
        Tab::Stats,
        Tab::Locks,
        Tab::Replication,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Overview => "Overview",
            Tab::Activity => "Activity",
            Tab::Queries => "Queries",
            Tab::Stats => "Stats",
            Tab::Locks => "Locks",
            Tab::Replication => "Replication",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|t| *t == self).unwrap_or(0)
    }

    pub fn next(self) -> Tab {
        let n = Self::ALL.len();
        Self::ALL[(self.index() + 1) % n]
    }

    pub fn prev(self) -> Tab {
        let n = Self::ALL.len();
        Self::ALL[(self.index() + n - 1) % n]
    }
}

pub struct App {
    pub pool: Pool,
    pub refresh_every: Duration,
    pub tab: Tab,
    pub paused: bool,
    pub last_refresh: Option<Instant>,
    pub should_quit: bool,
    pub show_help: bool,
    pub error: Option<String>,

    // Per-tab selected row index
    pub selected: [usize; 6],

    // Data
    pub overview: Option<db::overview::Overview>,
    pub sessions: Vec<db::activity::Session>,
    pub statements: Vec<db::statements::StatementRow>,
    pub db_stats: Vec<db::stats::DatabaseStat>,
    pub table_stats: Vec<db::stats::TableStat>,
    pub stats_show_tables: bool,
    pub locks: Vec<db::locks::LockWait>,
    pub replication: Option<db::replication::ReplicationInfo>,
}

impl App {
    pub fn new(pool: Pool, refresh_every: Duration) -> Self {
        Self {
            pool,
            refresh_every,
            tab: Tab::Overview,
            paused: false,
            last_refresh: None,
            should_quit: false,
            show_help: false,
            error: None,
            selected: [0; 6],
            overview: None,
            sessions: Vec::new(),
            statements: Vec::new(),
            db_stats: Vec::new(),
            table_stats: Vec::new(),
            stats_show_tables: false,
            locks: Vec::new(),
            replication: None,
        }
    }

    pub async fn run(mut self, terminal: &mut Tui) -> Result<()> {
        // Load initial data before entering the loop
        self.refresh_data().await;

        let mut events = EventStream::new();
        let mut ticker = tokio::time::interval(self.refresh_every);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        ticker.tick().await; // consume the immediate first tick

        while !self.should_quit {
            terminal.draw(|f| ui::draw(f, &self))?;

            tokio::select! {
                maybe_event = events.next() => {
                    match maybe_event {
                        Some(Ok(ev)) => {
                            if self.handle_event(ev) {
                                self.refresh_data().await;
                            }
                        }
                        Some(Err(_)) => continue,
                        None => break,
                    }
                }
                _ = ticker.tick() => {
                    if !self.paused {
                        self.refresh_data().await;
                    }
                }
            }
        }
        Ok(())
    }

    async fn refresh_data(&mut self) {
        let mut first_error: Option<String> = None;

        macro_rules! try_fetch {
            ($result:expr) => {
                match $result {
                    Ok(v) => Some(v),
                    Err(e) => {
                        if first_error.is_none() {
                            first_error = Some(e.to_string());
                        }
                        None
                    }
                }
            };
        }

        if let Some(v) = try_fetch!(db::overview::fetch(&self.pool).await) {
            self.overview = Some(v);
        }
        if let Some(v) = try_fetch!(db::activity::fetch(&self.pool).await) {
            self.sessions = v;
        }
        if let Some(v) = try_fetch!(db::statements::top_slow(&self.pool, 50).await) {
            self.statements = v;
        } else {
            // pg_stat_statements not installed is a common case — keep existing and clear error
            if let Some(ref e) = first_error {
                if e.contains("pg_stat_statements") {
                    first_error = None;
                }
            }
        }
        if let Some(v) = try_fetch!(db::stats::databases(&self.pool).await) {
            self.db_stats = v;
        }
        if let Some(v) = try_fetch!(db::stats::tables(&self.pool).await) {
            self.table_stats = v;
        }
        if let Some(v) = try_fetch!(db::locks::fetch(&self.pool).await) {
            self.locks = v;
        }
        if let Some(v) = try_fetch!(db::replication::fetch(&self.pool).await) {
            self.replication = Some(v);
        }

        self.error = first_error;
        self.last_refresh = Some(Instant::now());

        // Clamp selections so they don't go out of bounds after data changes
        for tab in Tab::ALL {
            let len = self.list_len(tab);
            let idx = tab.index();
            if len > 0 && self.selected[idx] >= len {
                self.selected[idx] = len - 1;
            }
        }
    }

    fn list_len(&self, tab: Tab) -> usize {
        match tab {
            Tab::Overview => 0,
            Tab::Activity => self.sessions.len(),
            Tab::Queries => self.statements.len(),
            Tab::Stats => {
                if self.stats_show_tables {
                    self.table_stats.len()
                } else {
                    self.db_stats.len()
                }
            }
            Tab::Locks => self.locks.len(),
            Tab::Replication => {
                self.replication.as_ref().map(|r| r.replicas.len()).unwrap_or(0)
            }
        }
    }

    fn current_list_len(&self) -> usize {
        self.list_len(self.tab)
    }

    // Returns true when a forced refresh is needed (r key).
    fn handle_event(&mut self, ev: Event) -> bool {
        let Event::Key(key) = ev else { return false };
        if key.kind != KeyEventKind::Press {
            return false;
        }

        // Any key dismisses the help overlay
        if self.show_help {
            self.show_help = false;
            return false;
        }

        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            (KeyCode::Esc, _) => {
                self.error = None;
            }
            (KeyCode::Char('?'), _) | (KeyCode::F(1), _) => {
                self.show_help = true;
            }
            (KeyCode::Tab, _) => self.tab = self.tab.next(),
            (KeyCode::BackTab, _) => self.tab = self.tab.prev(),
            (KeyCode::Char('1'), _) => self.tab = Tab::Overview,
            (KeyCode::Char('2'), _) => self.tab = Tab::Activity,
            (KeyCode::Char('3'), _) => self.tab = Tab::Queries,
            (KeyCode::Char('4'), _) => self.tab = Tab::Stats,
            (KeyCode::Char('5'), _) => self.tab = Tab::Locks,
            (KeyCode::Char('6'), _) => self.tab = Tab::Replication,
            (KeyCode::Char('p'), _) => self.paused = !self.paused,
            (KeyCode::Char('r'), _) => return true,
            (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                self.select_next();
            }
            (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                self.select_prev();
            }
            (KeyCode::Char('g'), KeyModifiers::NONE) => {
                self.selected[self.tab.index()] = 0;
            }
            (KeyCode::Char('G'), _) => {
                let len = self.current_list_len();
                if len > 0 {
                    self.selected[self.tab.index()] = len - 1;
                }
            }
            (KeyCode::PageDown, _) => {
                let idx = self.tab.index();
                let len = self.current_list_len();
                if len > 0 {
                    self.selected[idx] = (self.selected[idx] + 10).min(len - 1);
                }
            }
            (KeyCode::PageUp, _) => {
                let idx = self.tab.index();
                self.selected[idx] = self.selected[idx].saturating_sub(10);
            }
            (KeyCode::Char('s'), KeyModifiers::NONE) if self.tab == Tab::Stats => {
                self.stats_show_tables = !self.stats_show_tables;
                self.selected[Tab::Stats.index()] = 0;
            }
            _ => {}
        }
        false
    }

    fn select_next(&mut self) {
        let idx = self.tab.index();
        let len = self.current_list_len();
        if len > 0 {
            self.selected[idx] = (self.selected[idx] + 1).min(len - 1);
        }
    }

    fn select_prev(&mut self) {
        let idx = self.tab.index();
        if self.selected[idx] > 0 {
            self.selected[idx] -= 1;
        }
    }
}
