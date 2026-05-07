use clap::Parser;
use std::time::Duration;

/// pginsight — interactive PostgreSQL monitor for the terminal.
#[derive(Parser, Debug, Clone)]
#[command(name = "pginsight", author, version, about, long_about = None)]
pub struct Cli {
    /// Postgres host (skips the host prompt)
    #[arg(short = 'H', long, value_name = "HOST")]
    pub host: Option<String>,

    /// Postgres port (skips the port prompt)
    #[arg(short = 'p', long, value_name = "PORT")]
    pub port: Option<u16>,

    /// Postgres username (skips the username prompt)
    #[arg(short = 'U', long, value_name = "USER")]
    pub username: Option<String>,

    /// Database name (skips the database prompt)
    #[arg(short = 'd', long, value_name = "DBNAME")]
    pub dbname: Option<String>,

    /// Skip password prompt and connect without a password (peer / trust auth)
    #[arg(long)]
    pub no_password: bool,

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
