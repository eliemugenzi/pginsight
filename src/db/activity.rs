use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;

use super::Pool;

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub pid: i32,
    pub usename: Option<String>,
    pub datname: Option<String>,
    pub application_name: Option<String>,
    pub client_addr: Option<String>,
    pub state: Option<String>,
    pub wait_event_type: Option<String>,
    pub wait_event: Option<String>,
    pub query: Option<String>,
    pub query_start: Option<DateTime<Utc>>,
    pub duration_ms: f64,
}

pub async fn fetch(pool: &Pool) -> Result<Vec<Session>> {
    let client = pool.client().await;
    let rows = client
        .query(
            "SELECT
                pid,
                usename,
                datname,
                application_name,
                client_addr::text                                           AS client_addr,
                state,
                wait_event_type,
                wait_event,
                query,
                query_start,
                COALESCE(
                    EXTRACT(EPOCH FROM (now() - query_start)) * 1000.0,
                    0.0)                                                    AS duration_ms
             FROM pg_stat_activity
             WHERE pid <> pg_backend_pid()
               AND backend_type = 'client backend'
             ORDER BY duration_ms DESC NULLS LAST",
            &[],
        )
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| Session {
            pid: row.get("pid"),
            usename: row.try_get("usename").ok(),
            datname: row.try_get("datname").ok(),
            application_name: row.try_get("application_name").ok(),
            client_addr: row.try_get("client_addr").ok(),
            state: row.try_get("state").ok(),
            wait_event_type: row.try_get("wait_event_type").ok(),
            wait_event: row.try_get("wait_event").ok(),
            query: row.try_get("query").ok(),
            query_start: row.try_get("query_start").ok(),
            duration_ms: row.get("duration_ms"),
        })
        .collect())
}

/// Cancel a backend by PID (non-destructive — current query is cancelled).
pub async fn cancel_backend(pool: &Pool, pid: i32) -> Result<bool> {
    let client = pool.client().await;
    let row = client
        .query_one("SELECT pg_cancel_backend($1)", &[&pid])
        .await?;
    Ok(row.get(0))
}

/// Terminate a backend by PID (forceful — connection is closed).
pub async fn terminate_backend(pool: &Pool, pid: i32) -> Result<bool> {
    let client = pool.client().await;
    let row = client
        .query_one("SELECT pg_terminate_backend($1)", &[&pid])
        .await?;
    Ok(row.get(0))
}
