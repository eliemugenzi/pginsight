//! Slow / hot queries from the `pg_stat_statements` extension.
//!
//! TODO (Claude Code): support both modern (1.8+) and legacy column names,
//! add filtering, and an `EXPLAIN` helper.

use anyhow::{anyhow, Result};
use serde::Serialize;

use super::Pool;

#[derive(Debug, Clone, Serialize)]
pub struct StatementRow {
    pub calls: i64,
    pub total_exec_ms: f64,
    pub mean_exec_ms: f64,
    pub rows: i64,
    pub query: String,
}

/// Top N statements by total execution time.
pub async fn top_slow(pool: &Pool, limit: u32) -> Result<Vec<StatementRow>> {
    let client = pool.client().await;
    let installed: bool = client
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'pg_stat_statements')",
            &[],
        )
        .await?
        .get(0);
    if !installed {
        return Err(anyhow!(
            "pg_stat_statements is not installed. Run: CREATE EXTENSION pg_stat_statements;"
        ));
    }

    let rows = client
        .query(
            "SELECT calls, total_exec_time AS total_exec_ms,
                    mean_exec_time  AS mean_exec_ms,
                    rows, query
             FROM pg_stat_statements
             ORDER BY total_exec_time DESC
             LIMIT $1::bigint",
            &[&(limit as i64)],
        )
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| StatementRow {
            calls: row.get("calls"),
            total_exec_ms: row.get("total_exec_ms"),
            mean_exec_ms: row.get("mean_exec_ms"),
            rows: row.get("rows"),
            query: row.get("query"),
        })
        .collect())
}
