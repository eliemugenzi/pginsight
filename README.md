# pgmon

Interactive terminal UI for monitoring PostgreSQL — boilerplate.

This repo is a Rust scaffold: dependencies are wired up, the project builds,
the TUI launches with tabs and key handling, and there's a stub `db::*`
module per monitoring view. Real query logic and tab rendering are left as
TODOs to be filled in.

## Quick start

```bash
# 1. Make sure you have a recent Rust toolchain (>= 1.75 for ratatui 0.28).
rustup update stable

# 2. Point at a Postgres database.
export DATABASE_URL=postgres://localhost/postgres

# 3. Build & run the TUI.
cargo run --release
```

Run `pgmon --help` for flags. Logs go to stderr — set `PGMON_LOG=info` to see
postgres errors without clobbering the TUI on stdout.

## What works today

- `cargo build` succeeds.
- `pgmon --url postgres://...` connects (TLS-less for now) and opens the TUI.
- Tab navigation: `1`–`6`, `Tab` / `Shift+Tab`.
- `q` / `Ctrl+C` quit · `r` request refresh · `p` pause auto-refresh.
- Each tab renders a placeholder panel with a TODO note pointing at the
  matching `db::*` module.
- Each `db::*` module already runs a minimal real query against Postgres,
  so plumbing is verified end-to-end.

## What's left (per tab)

| Tab          | DB module           | Build out                                                   |
|--------------|---------------------|-------------------------------------------------------------|
| Overview     | `db::overview`      | version, uptime, conns, cache hit %, TPS, deadlocks         |
| Activity     | `db::activity`      | live `pg_stat_activity` table; cancel / terminate keys      |
| Queries      | `db::statements`    | top slow queries from `pg_stat_statements`; EXPLAIN; reset  |
| Stats        | `db::stats`         | DB sizes, table bloat, index usage, vacuum freshness        |
| Locks        | `db::locks`         | wait graph using `pg_blocking_pids` + lock mode/relation    |
| Replication  | `db::replication`   | primary/replica state, replicas, slots, WAL retention       |

Things to add cross-cutting:

- Reconnect on transient errors (the `Pool::reconnect` hook is already there).
- TLS support — drop in `tokio-postgres-rustls` and route `sslmode` from the URL.
- CSV/JSON export of the current tab (`o` keybinding suggested in `app.rs`).
- Non-interactive subcommands (`pgmon slow`, `pgmon activity`, ...) — see
  `src/export.rs` placeholder.
- Theme / color customization.
- An EXPLAIN overlay for the Queries tab.

## Layout

```
src/
├── main.rs            entry point: parse CLI, build Pool, run App
├── cli.rs             clap arguments
├── tui.rs             terminal init / restore + panic hook
├── format.rs          tiny formatting helpers (bytes / duration / pct)
├── app.rs             App state, event loop, tab navigation
├── db/
│   ├── mod.rs         Pool wrapper around tokio_postgres::Client
│   ├── overview.rs    server overview (stub)
│   ├── activity.rs    pg_stat_activity (stub)
│   ├── statements.rs  pg_stat_statements (stub)
│   ├── stats.rs       db & table stats (stub)
│   ├── locks.rs       lock waits (stub)
│   └── replication.rs replication & WAL (stub)
├── ui/
│   ├── mod.rs         layout, tab bar, status bar
│   ├── overview.rs    Overview tab (stub)
│   ├── activity.rs    Activity tab (stub)
│   ├── queries.rs     Queries tab (stub)
│   ├── stats.rs       Stats tab (stub)
│   ├── locks.rs       Locks tab (stub)
│   └── replication.rs Replication tab (stub)
└── export.rs          reserved for non-interactive formatters
```

## Required Postgres extensions

The full version will lean on:

- `pg_stat_statements` for the Queries tab — add it to
  `shared_preload_libraries` and `CREATE EXTENSION pg_stat_statements;`.

The other tabs use built-in catalogs and need no extras.

## Notes for Claude Code

- Each `db::*` stub returns a small `Serialize`-able struct already — extend
  the struct, extend the SQL, and you're done. The `Pool::client()` method
  hands you an `Arc<Client>` you can use directly.
- Each `ui::*` tab gets `&App` and a `Rect`. Add per-tab state to `App` if
  you need selection/sort/filter (e.g. `TableState`, sort key enums, etc.).
- The refresh loop in `App::run` already ticks every `--refresh` seconds.
  Hook your data-fetch in the marked TODO inside the `ticker.tick()` arm.
- Errors should bubble up via `anyhow::Result`. Surface them in the TUI by
  storing `error: Option<String>` on `App` and rendering an overlay.
