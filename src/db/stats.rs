use anyhow::Result;
use serde::Serialize;

use super::Pool;

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseStat {
    pub datname: String,
    pub size_bytes: i64,
    pub connections: i64,
    pub cache_hit_ratio: Option<f64>,
    pub xact_commit: i64,
    pub xact_rollback: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TableStat {
    pub schema: String,
    pub name: String,
    pub total_size_bytes: i64,
    pub n_live_tup: i64,
    pub n_dead_tup: i64,
    pub seq_scan: i64,
    pub idx_scan: i64,
}

pub async fn databases(pool: &Pool) -> Result<Vec<DatabaseStat>> {
    let client = pool.client().await;
    let rows = client
        .query(
            "SELECT
                d.datname,
                pg_database_size(d.datname)::bigint                             AS size_bytes,
                COALESCE(s.numbackends, 0)::bigint                              AS connections,
                CASE WHEN COALESCE(s.blks_hit, 0) + COALESCE(s.blks_read, 0) = 0 THEN NULL
                     ELSE s.blks_hit::float / (s.blks_hit + s.blks_read) END   AS cache_hit_ratio,
                COALESCE(s.xact_commit, 0)::bigint                              AS xact_commit,
                COALESCE(s.xact_rollback, 0)::bigint                            AS xact_rollback
             FROM pg_database d
             LEFT JOIN pg_stat_database s ON s.datname = d.datname
             WHERE NOT d.datistemplate
             ORDER BY pg_database_size(d.datname) DESC",
            &[],
        )
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| DatabaseStat {
            datname: r.get("datname"),
            size_bytes: r.get("size_bytes"),
            connections: r.get("connections"),
            cache_hit_ratio: r.try_get("cache_hit_ratio").ok(),
            xact_commit: r.get("xact_commit"),
            xact_rollback: r.get("xact_rollback"),
        })
        .collect())
}

pub async fn tables(pool: &Pool) -> Result<Vec<TableStat>> {
    let client = pool.client().await;
    let rows = client
        .query(
            "SELECT
                n.nspname                                   AS schema,
                c.relname                                   AS name,
                pg_total_relation_size(c.oid)::bigint       AS total_size_bytes,
                COALESCE(s.n_live_tup, 0)::bigint           AS n_live_tup,
                COALESCE(s.n_dead_tup, 0)::bigint           AS n_dead_tup,
                COALESCE(s.seq_scan, 0)::bigint             AS seq_scan,
                COALESCE(s.idx_scan, 0)::bigint             AS idx_scan
             FROM pg_class c
             JOIN pg_namespace n ON n.oid = c.relnamespace
             LEFT JOIN pg_stat_user_tables s ON s.relid = c.oid
             WHERE c.relkind = 'r'
               AND n.nspname NOT IN ('pg_catalog', 'information_schema')
             ORDER BY pg_total_relation_size(c.oid) DESC
             LIMIT 100",
            &[],
        )
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| TableStat {
            schema: r.get("schema"),
            name: r.get("name"),
            total_size_bytes: r.get("total_size_bytes"),
            n_live_tup: r.get("n_live_tup"),
            n_dead_tup: r.get("n_dead_tup"),
            seq_scan: r.get("seq_scan"),
            idx_scan: r.get("idx_scan"),
        })
        .collect())
}
