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
///
/// Detects the correct column names at runtime so it works across all
/// pg_stat_statements versions:
///   ≥ 1.8 (PostgreSQL ≥ 13): total_exec_time / mean_exec_time
///   ≤ 1.7 (PostgreSQL ≤ 12): total_time / mean_time
pub async fn top_slow(pool: &Pool, limit: u32) -> Result<Vec<StatementRow>> {
    let client = pool.client().await;

    // Check the extension is installed in the current database.
    let installed: bool = client
        .query_one(
            "SELECT EXISTS(
                SELECT 1 FROM pg_extension WHERE extname = 'pg_stat_statements'
             )",
            &[],
        )
        .await?
        .get(0);

    if !installed {
        return Err(anyhow!(
            "pg_stat_statements extension is not installed in this database.\n\
             Run: CREATE EXTENSION pg_stat_statements;"
        ));
    }

    // Discover the actual time column name — handles all versions without guessing.
    let time_col: Option<String> = client
        .query_opt(
            "SELECT column_name::text
             FROM information_schema.columns
             WHERE table_name = 'pg_stat_statements'
               AND column_name IN ('total_exec_time', 'total_time')
             ORDER BY CASE column_name
                          WHEN 'total_exec_time' THEN 1  -- prefer modern name
                          ELSE 2
                      END
             LIMIT 1",
            &[],
        )
        .await?
        .map(|r| r.get(0));

    let time_col = time_col.ok_or_else(|| {
        anyhow!(
            "pg_stat_statements is installed but no recognised time column \
             (total_exec_time / total_time) was found. \
             Try: ALTER EXTENSION pg_stat_statements UPDATE;"
        )
    })?;

    let mean_col = if time_col == "total_exec_time" {
        "mean_exec_time"
    } else {
        "mean_time"
    };

    // Build the query with the discovered column names.
    // time_col / mean_col come from information_schema — not user input — so this is safe.
    let sql = format!(
        "SELECT calls,
                {time_col}::float8 AS total_exec_ms,
                {mean_col}::float8 AS mean_exec_ms,
                rows::bigint,
                query
         FROM pg_stat_statements
         ORDER BY {time_col} DESC
         LIMIT $1::bigint"
    );

    let rows = client.query(&sql, &[&(limit as i64)]).await?;

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
