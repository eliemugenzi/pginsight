# pginsight

**pginsight** is an interactive terminal UI for monitoring PostgreSQL in real time. Connect to any database — local or remote — and immediately get visibility into performance, active sessions, slow queries, locks, and replication health, all from your terminal.

---

## Installation

### Homebrew (macOS) — recommended

```bash
brew tap eliemugenzi/pginsight
brew install pginsight
```

### Download binary

Grab the latest universal binary (Apple Silicon + Intel) from the
[Releases](https://github.com/eliemugenzi/pginsight/releases) page.

```bash
# Extract and install
tar -xzf pginsight-*-macos-universal.tar.gz
mv pginsight /usr/local/bin/
```

> **First launch on macOS:** if Gatekeeper blocks the binary, run once:
> ```bash
> xattr -dr com.apple.quarantine /usr/local/bin/pginsight
> ```

### Build from source

```bash
cargo install --path .
```

> Requires Rust 1.75+ and a reachable PostgreSQL instance.

---

## Connecting

pginsight prompts you for credentials on startup — no connection string required:

```
  pginsight  PostgreSQL Monitor
  ─────────────────────────────────────────────────

  Host        [localhost]
  Port        [5432]
  Username    [youruser]
  Password
  Database    [youruser]
```

CLI flags let you pre-fill any field and skip that prompt:

```bash
# Pre-fill individual fields — only the missing ones will be prompted
pginsight -H myserver -U admin -d production

# Skip the password prompt for local peer / trust auth
pginsight --no-password
```

---

## What you get

### Overview
Server version, role (primary or replica), uptime, a live connection usage gauge, cache hit ratio gauge, and a breakdown of active, idle, and idle-in-transaction sessions.

### Activity
A live view of `pg_stat_activity` — every client session with its state, wait events, duration, and current query. Filter sessions by state with `f`:

- **all** — every connected client
- **active** — sessions currently executing a query
- **idle** — connected but doing nothing
- **idle in transaction** — inside an open transaction (worth watching)
- **waiting** — blocked on a lock or resource

### Queries
Top 50 slowest statements from `pg_stat_statements`, ranked by total execution time. Selecting a row shows the full query with syntax highlighting and clause-level formatting. Requires `pg_stat_statements` (see below).

### Stats
Database-level and table-level statistics in one place. Toggle between views with `s`:

- **Databases** — size, connections, cache hit ratio, commits, and rollbacks per database
- **Tables** — size, live rows, dead rows (with bloat % coloring), and sequential vs index scan counts

### Locks
Every blocked session paired with the session blocking it — lock mode, relation name, and how long it has been waiting. Selecting a row shows the full SQL of both the blocked and blocking query, highlighted side by side.

### Replication
Primary/replica role, current WAL LSN, connected replicas with write/flush/replay lag, and replication slot retention.

---

## Key bindings

| Key | Action |
|-----|--------|
| `1` – `6` | Jump to tab |
| `Tab` / `Shift+Tab` | Next / previous tab |
| `j` / `k` or `↑` / `↓` | Navigate rows |
| `g` / `G` | Jump to top / bottom |
| `PgUp` / `PgDn` | Page through rows |
| `f` | Cycle session filter (Activity tab) |
| `s` | Toggle Databases ↔ Tables (Stats tab) |
| `r` | Force refresh now |
| `p` | Pause / resume auto-refresh |
| `?` / `F1` | Help overlay |
| `Esc` | Dismiss error |
| `q` / `Ctrl+C` | Quit |

Refresh interval defaults to 2 seconds. Change it with `--refresh <secs>`.

---

## pg_stat_statements

The **Queries** tab requires the `pg_stat_statements` extension. If it is not set up, the tab will tell you exactly what to do.

```sql
-- 1. Add to postgresql.conf and restart PostgreSQL:
shared_preload_libraries = 'pg_stat_statements'

-- 2. After restart, run once per database:
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
```

All other tabs use built-in PostgreSQL system catalogs and need no extensions.

---

## License

MIT
