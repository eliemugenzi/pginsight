use clap::Parser;
use std::time::Duration;

/// pginsight — interactive PostgreSQL monitor for the terminal.
#[derive(Parser, Debug, Clone)]
#[command(name = "pginsight", author, version, about, long_about = None)]
pub struct Cli {
    /// Postgres host (overrides interactive prompt; env: PGHOST)
    #[arg(short = 'H', long, env = "PGHOST", value_name = "HOST")]
    pub host: Option<String>,

    /// Postgres port (overrides interactive prompt; env: PGPORT)
    #[arg(short = 'p', long, env = "PGPORT", value_name = "PORT")]
    pub port: Option<u16>,

    /// Postgres username (overrides interactive prompt; env: PGUSER)
    #[arg(short = 'U', long, env = "PGUSER", value_name = "USER")]
    pub username: Option<String>,

    /// Database name (overrides interactive prompt; env: PGDATABASE)
    #[arg(short = 'd', long, env = "PGDATABASE", value_name = "DBNAME")]
    pub dbname: Option<String>,

    /// Skip password prompt and connect without a password (peer / trust auth)
    #[arg(long)]
    pub no_password: bool,

    /// Auto-accept all defaults — skip interactive prompt entirely
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Refresh interval in seconds
    #[arg(short = 'r', long = "refresh", default_value_t = 2, value_name = "SECS")]
    pub refresh_secs: u64,

    /// Application name reported to Postgres (visible in pg_stat_activity)
    #[arg(long = "app-name", default_value = "pginsight")]
    pub app_name: String,
}

impl Cli {
    pub fn refresh_interval(&self) -> Duration {
        Duration::from_secs(self.refresh_secs.max(1))
    }
}
