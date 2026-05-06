use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;

use super::Pool;

#[derive(Debug, Clone, Default, Serialize)]
pub struct Overview {
    pub server_version: String,
    pub current_database: String,
    pub current_user: String,
    pub started_at: Option<DateTime<Utc>>,
    pub total_connections: i64,
    pub active_connections: i64,
    pub idle_connections: i64,
    pub idle_in_tx_connections: i64,
    pub max_connections: i64,
    pub cache_hit_ratio: f64,
    pub is_in_recovery: bool,
    pub xact_commit: i64,
    pub xact_rollback: i64,
    pub db_size_bytes: i64,
}

pub async fn fetch(pool: &Pool) -> Result<Overview> {
    let client = pool.client().await;
    let row = client
        .query_one(
            "SELECT
                current_setting('server_version')                           AS server_version,
                current_database()                                          AS db,
                current_user                                                AS usr,
                pg_postmaster_start_time()                                  AS started_at,
                pg_is_in_recovery()                                         AS in_recovery,
                (SELECT count(*)::bigint FROM pg_stat_activity
                 WHERE backend_type = 'client backend')                     AS total_connections,
                (SELECT count(*)::bigint FROM pg_stat_activity
                 WHERE backend_type = 'client backend'
                   AND state = 'active')                                    AS active_connections,
                (SELECT count(*)::bigint FROM pg_stat_activity
                 WHERE backend_type = 'client backend'
                   AND state = 'idle')                                      AS idle_connections,
                (SELECT count(*)::bigint FROM pg_stat_activity
                 WHERE backend_type = 'client backend'
                   AND state LIKE 'idle in transaction%')                   AS idle_in_tx_connections,
                current_setting('max_connections')::bigint                  AS max_connections,
                COALESCE(
                    (SELECT blks_hit::float / NULLIF(blks_hit + blks_read, 0)
                     FROM pg_stat_database WHERE datname = current_database()),
                    1.0)                                                    AS cache_hit_ratio,
                COALESCE(
                    (SELECT xact_commit::bigint FROM pg_stat_database
                     WHERE datname = current_database()), 0)                AS xact_commit,
                COALESCE(
                    (SELECT xact_rollback::bigint FROM pg_stat_database
                     WHERE datname = current_database()), 0)                AS xact_rollback,
                pg_database_size(current_database())::bigint                AS db_size_bytes",
            &[],
        )
        .await?;

    Ok(Overview {
        server_version: row.get("server_version"),
        current_database: row.get("db"),
        current_user: row.get("usr"),
        started_at: row.try_get("started_at").ok(),
        is_in_recovery: row.get("in_recovery"),
        total_connections: row.get("total_connections"),
        active_connections: row.get("active_connections"),
        idle_connections: row.get("idle_connections"),
        idle_in_tx_connections: row.get("idle_in_tx_connections"),
        max_connections: row.get("max_connections"),
        cache_hit_ratio: row.get("cache_hit_ratio"),
        xact_commit: row.get("xact_commit"),
        xact_rollback: row.get("xact_rollback"),
        db_size_bytes: row.get("db_size_bytes"),
    })
}
