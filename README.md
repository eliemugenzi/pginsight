# pginsight

Interactive terminal UI for monitoring PostgreSQL — built with Rust.

## Quick start

```bash
# Build & run
cargo run --release

# pginsight will prompt for connection details:
#   Host     [localhost]
#   Port     [5432]
#   Username [youruser]
#   Password
#   Database [youruser]
```

You can also pre-fill any field via CLI flags or standard PostgreSQL env vars:

```bash
# CLI flags (any combination)
pginsight -H myserver -U admin -d production

# Env vars (same as psql)
PGHOST=myserver PGUSER=admin PGDATABASE=prod PGPASSWORD=secret pginsight

# Skip password prompt (local peer / trust auth)
pginsight --no-password

# Auto-accept all defaults, no prompts (useful in scripts)
pginsight -y
```

Set `PGINSIGHT_LOG=info` to send debug logs to stderr (they won't corrupt the TUI).

## Features

| Tab          | What you see                                                          |
|--------------|-----------------------------------------------------------------------|
| Overview     | Server version, role (primary/replica), uptime, connection gauge, cache-hit gauge, commit/rollback counts |
| Activity     | Live `pg_stat_activity` — state, wait events, duration, query detail  |
| Queries      | Top-50 slowest statements from `pg_stat_statements`                   |
| Stats        | Database sizes + cache hit; table sizes, live/dead rows, seq/idx scans (toggle with `s`) |
| Locks        | Blocked/blocking session pairs — lock mode, relation, wait time       |
| Replication  | Replica list with write/flush/replay lag; replication slots           |

## Key bindings

| Key | Action |
|-----|--------|
| `1`–`6` | Jump to tab |
| `Tab` / `Shift+Tab` | Next / previous tab |
| `j` / `k` or `↑` / `↓` | Navigate rows |
| `g` / `G` | Jump to top / bottom |
| `PgUp` / `PgDn` | Page through rows |
| `s` | Toggle Stats: Databases ↔ Tables |
| `r` | Force refresh now |
| `p` | Pause / resume auto-refresh |
| `?` / `F1` | Help overlay |
| `Esc` | Dismiss error |
| `q` / `Ctrl+C` | Quit |

## Required Postgres extensions

The **Queries** tab requires `pg_stat_statements`:

```sql
-- Add to postgresql.conf:
shared_preload_libraries = 'pg_stat_statements'

-- Then after restart:
CREATE EXTENSION pg_stat_statements;
```

All other tabs use built-in system catalogs — no extensions needed.

## Layout

```
src/
├── main.rs            entry point
├── cli.rs             clap arguments (host, port, user, dbname flags)
├── connect.rs         interactive credential prompt
├── tui.rs             terminal init / restore + panic hook
├── format.rs          bytes / ms / number / datetime formatters
├── app.rs             App state, event loop, tab navigation, data refresh
├── db/
│   ├── mod.rs         Pool (from_credentials + reconnect)
│   ├── overview.rs    server overview query
│   ├── activity.rs    pg_stat_activity + cancel/terminate helpers
│   ├── statements.rs  pg_stat_statements top-N slow
│   ├── stats.rs       database & table statistics
│   ├── locks.rs       lock wait graph
│   └── replication.rs replication state, replicas, slots
└── ui/
    ├── mod.rs         layout, tab bar, status bar, help overlay
    ├── help.rs        keybinding reference overlay
    ├── overview.rs    Overview tab
    ├── activity.rs    Activity tab
    ├── queries.rs     Queries tab
    ├── stats.rs       Stats tab (two-mode)
    ├── locks.rs       Locks tab
    └── replication.rs Replication tab
```
