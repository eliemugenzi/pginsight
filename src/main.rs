mod app;
mod cli;
mod connect;
mod db;
mod export;
mod format;
mod sql_format;
mod tui;
mod ui;

use anyhow::{Context, Result};
use clap::Parser;

use crate::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();

    // Prompt for (or auto-fill) connection credentials.
    let creds = connect::prompt(&cli)?;

    let pool = db::Pool::from_credentials(
        &creds.host,
        creds.port,
        &creds.user,
        creds.password.as_deref(),
        &creds.dbname,
        &cli.app_name,
    )
    .await
    .with_context(|| {
        format!(
            "connecting to {}@{}:{}/{}",
            creds.user, creds.host, creds.port, creds.dbname
        )
    })?;

    let mut terminal = tui::init()?;
    let result = app::App::new(pool, cli.refresh_interval())
        .run(&mut terminal)
        .await;
    tui::restore()?;
    result
}

fn init_tracing() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = EnvFilter::try_from_env("PGINSIGHT_LOG").unwrap_or_else(|_| EnvFilter::new("warn"));
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(std::io::stderr))
        .try_init();
}
