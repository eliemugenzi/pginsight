use anyhow::Result;
use serde::Serialize;

use super::Pool;

#[derive(Debug, Clone, Serialize)]
pub struct LockWait {
    pub blocked_pid: i32,
    pub blocked_user: Option<String>,
    pub blocking_pid: i32,
    pub blocking_user: Option<String>,
    pub blocked_query: Option<String>,
    pub blocking_query: Option<String>,
    pub lock_mode: Option<String>,
    pub relation: Option<String>,
    pub wait_ms: f64,
}

pub async fn fetch(pool: &Pool) -> Result<Vec<LockWait>> {
    let client = pool.client().await;
    let rows = client
        .query(
            "SELECT DISTINCT ON (blocked.pid, blocking.pid)
                blocked.pid                                                     AS blocked_pid,
                blocked.usename                                                 AS blocked_user,
                blocking.pid                                                    AS blocking_pid,
                blocking.usename                                                AS blocking_user,
                blocked.query                                                   AS blocked_query,
                blocking.query                                                  AS blocking_query,
                bl.mode                                                         AS lock_mode,
                COALESCE(c.relname, '')                                         AS relation,
                COALESCE(
                    EXTRACT(EPOCH FROM (now() - blocked.query_start)) * 1000.0,
                    0.0)                                                        AS wait_ms
             FROM pg_stat_activity AS blocked
             JOIN pg_stat_activity AS blocking
               ON blocking.pid = ANY(pg_blocking_pids(blocked.pid))
             JOIN pg_locks bl
               ON bl.pid = blocked.pid AND NOT bl.granted
             LEFT JOIN pg_class c ON c.oid = bl.relation
             WHERE cardinality(pg_blocking_pids(blocked.pid)) > 0
             ORDER BY blocked.pid, blocking.pid, bl.mode",
            &[],
        )
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| LockWait {
            blocked_pid: r.get("blocked_pid"),
            blocked_user: r.try_get("blocked_user").ok(),
            blocking_pid: r.get("blocking_pid"),
            blocking_user: r.try_get("blocking_user").ok(),
            blocked_query: r.try_get("blocked_query").ok(),
            blocking_query: r.try_get("blocking_query").ok(),
            lock_mode: r.try_get("lock_mode").ok(),
            relation: r.try_get("relation").ok(),
            wait_ms: r.get("wait_ms"),
        })
        .collect())
}
