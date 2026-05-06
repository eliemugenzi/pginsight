//! Postgres connection layer.
//!
//! Each submodule (`overview`, `activity`, `statements`, `stats`, `locks`,
//! `replication`) owns one logical "view" of the database. They each expose a
//! stub `fetch` function with the right signature; fill them in with real SQL.
//!
//! TLS is intentionally not wired up in this boilerplate — add a connector
//! (e.g. `tokio-postgres-rustls`) to `build_client` when you need it.

pub mod activity;
pub mod locks;
pub mod overview;
pub mod replication;
pub mod statements;
pub mod stats;

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_postgres::{Client, Config, NoTls};

/// Thin wrapper around a single Postgres `Client`. Holds the parsed `Config`
/// so callers can `reconnect()` on transient failures.
#[derive(Clone)]
pub struct Pool {
    config: Config,
    inner: Arc<RwLock<Arc<Client>>>,
}

impl Pool {
    /// Connect using individual credential fields (no URL encoding needed).
    pub async fn from_credentials(
        host: &str,
        port: u16,
        user: &str,
        password: Option<&str>,
        dbname: &str,
        app_name: &str,
    ) -> Result<Self> {
        let mut config = Config::new();
        config.host(host);
        config.port(port);
        config.user(user);
        config.dbname(dbname);
        config.application_name(app_name);
        if let Some(pw) = password {
            config.password(pw);
        }
        let client = build_client(&config).await?;
        Ok(Self {
            config,
            inner: Arc::new(RwLock::new(Arc::new(client))),
        })
    }

    /// Open a new connection from a postgres:// URL (kept for power users).
    #[allow(dead_code)]
    pub async fn connect(url: &str, app_name: &str) -> Result<Self> {
        let mut config: Config = url
            .parse()
            .with_context(|| format!("parsing connection URL `{}`", redact_url(url)))?;
        config.application_name(app_name);

        let client = build_client(&config).await?;
        Ok(Self {
            config,
            inner: Arc::new(RwLock::new(Arc::new(client))),
        })
    }

    /// Get a cloneable handle to the current `Client`.
    pub async fn client(&self) -> Arc<Client> {
        self.inner.read().await.clone()
    }

    /// Replace the underlying client with a fresh connection.
    pub async fn reconnect(&self) -> Result<()> {
        let client = build_client(&self.config).await?;
        *self.inner.write().await = Arc::new(client);
        Ok(())
    }

    pub fn host(&self) -> String {
        self.config
            .get_hosts()
            .iter()
            .map(|h| match h {
                tokio_postgres::config::Host::Tcp(s) => s.clone(),
                #[allow(unreachable_patterns)]
                _ => "<unix>".into(),
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn port(&self) -> u16 {
        self.config.get_ports().first().copied().unwrap_or(5432)
    }

    pub fn dbname(&self) -> &str {
        self.config.get_dbname().unwrap_or("?")
    }

    pub fn user(&self) -> &str {
        self.config.get_user().unwrap_or("?")
    }
}

async fn build_client(config: &Config) -> Result<Client> {
    let (client, conn) = config
        .connect(NoTls)
        .await
        .context("opening connection to Postgres")?;
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            tracing::error!("postgres connection terminated: {e}");
        }
    });
    Ok(client)
}

/// Strip credentials from a URL before logging.
pub fn redact_url(url: &str) -> String {
    // postgres://user:pass@host:port/db -> postgres://user@host:port/db
    if let Some(scheme_end) = url.find("://") {
        let (scheme, rest) = url.split_at(scheme_end + 3);
        if let Some(at) = rest.find('@') {
            let creds = &rest[..at];
            let after = &rest[at..];
            if let Some(colon) = creds.find(':') {
                let user = &creds[..colon];
                return format!("{scheme}{user}{after}");
            }
        }
    }
    url.to_string()
}
