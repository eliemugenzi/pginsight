use anyhow::Result;
use serde::Serialize;

use super::Pool;

#[derive(Debug, Clone, Serialize)]
pub struct Replica {
    pub pid: i32,
    pub application_name: Option<String>,
    pub client_addr: Option<String>,
    pub state: Option<String>,
    pub sync_state: Option<String>,
    pub sent_lsn: Option<String>,
    pub write_lsn: Option<String>,
    pub flush_lsn: Option<String>,
    pub replay_lsn: Option<String>,
    pub write_lag_ms: Option<f64>,
    pub flush_lag_ms: Option<f64>,
    pub replay_lag_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplicationSlot {
    pub slot_name: String,
    pub plugin: Option<String>,
    pub slot_type: String,
    pub active: bool,
    pub retained_bytes: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ReplicationInfo {
    pub is_in_recovery: bool,
    pub current_wal_lsn: Option<String>,
    pub receive_lsn: Option<String>,
    pub replicas: Vec<Replica>,
    pub slots: Vec<ReplicationSlot>,
}

pub async fn fetch(pool: &Pool) -> Result<ReplicationInfo> {
    let client = pool.client().await;

    let row = client
        .query_one(
            "SELECT
                pg_is_in_recovery()                                         AS in_recovery,
                CASE WHEN NOT pg_is_in_recovery()
                     THEN pg_current_wal_lsn()::text END                    AS current_lsn,
                CASE WHEN pg_is_in_recovery()
                     THEN pg_last_wal_receive_lsn()::text END               AS receive_lsn",
            &[],
        )
        .await?;

    let mut info = ReplicationInfo {
        is_in_recovery: row.get("in_recovery"),
        current_wal_lsn: row.try_get("current_lsn").ok(),
        receive_lsn: row.try_get("receive_lsn").ok(),
        replicas: Vec::new(),
        slots: Vec::new(),
    };

    let replica_rows = client
        .query(
            "SELECT
                pid,
                application_name,
                client_addr::text                                           AS client_addr,
                state,
                sync_state,
                sent_lsn::text,
                write_lsn::text,
                flush_lsn::text,
                replay_lsn::text,
                EXTRACT(EPOCH FROM write_lag)::float8   * 1000.0            AS write_lag_ms,
                EXTRACT(EPOCH FROM flush_lag)::float8   * 1000.0            AS flush_lag_ms,
                EXTRACT(EPOCH FROM replay_lag)::float8  * 1000.0            AS replay_lag_ms
             FROM pg_stat_replication
             ORDER BY application_name",
            &[],
        )
        .await
        .unwrap_or_default();

    info.replicas = replica_rows
        .into_iter()
        .map(|r| Replica {
            pid: r.get("pid"),
            application_name: r.try_get("application_name").ok(),
            client_addr: r.try_get("client_addr").ok(),
            state: r.try_get("state").ok(),
            sync_state: r.try_get("sync_state").ok(),
            sent_lsn: r.try_get("sent_lsn").ok(),
            write_lsn: r.try_get("write_lsn").ok(),
            flush_lsn: r.try_get("flush_lsn").ok(),
            replay_lsn: r.try_get("replay_lsn").ok(),
            write_lag_ms: r.try_get("write_lag_ms").ok(),
            flush_lag_ms: r.try_get("flush_lag_ms").ok(),
            replay_lag_ms: r.try_get("replay_lag_ms").ok(),
        })
        .collect();

    let slot_rows = client
        .query(
            "SELECT
                slot_name,
                plugin,
                slot_type::text,
                active,
                CASE WHEN NOT pg_is_in_recovery()
                     THEN pg_wal_lsn_diff(
                              pg_current_wal_lsn(),
                              COALESCE(confirmed_flush_lsn, restart_lsn))::bigint
                END AS retained_bytes
             FROM pg_replication_slots
             ORDER BY slot_name",
            &[],
        )
        .await
        .unwrap_or_default();

    info.slots = slot_rows
        .into_iter()
        .map(|r| ReplicationSlot {
            slot_name: r.get("slot_name"),
            plugin: r.try_get("plugin").ok(),
            slot_type: r.get("slot_type"),
            active: r.get("active"),
            retained_bytes: r.try_get("retained_bytes").ok(),
        })
        .collect();

    Ok(info)
}
