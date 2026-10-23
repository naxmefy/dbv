# 0004. Async runtime and toolchain

Date: 2026-10-23

Status: Accepted

## Context

The TUI must never freeze while a query runs. Later milestones add network drivers (Postgres, MySQL) and query cancellation, which are naturally async, while the first driver (`rusqlite`) is blocking. We also need to fix the language edition and minimum supported Rust version before any product code exists.

## Decision

- **Runtime:** tokio. The TUI event loop receives terminal events and database results over a channel. Database calls run as tasks and report back with a message; the UI thread never blocks on a database.
- **Blocking drivers:** the SQLite driver wraps its `rusqlite` calls in `tokio::task::spawn_blocking`.
- **Traits:** the driver traits are `async` and must be usable as `dyn` objects (`Box<dyn Session>`, `Arc<dyn StatementLog>`). We use the `async-trait` crate for this, because native `async fn` in traits is not dyn-compatible.
- **Toolchain:** Rust edition 2024, Cargo resolver 3, toolchain `stable` pinned in `rust-toolchain.toml`.
- **MSRV:** 1.88, dictated by ratatui 0.30. It is checked in a separate CI job.

## Consequences

- Network drivers fit in naturally, and cancellation can be added later without redesigning the UI.
- Blocking drivers cost a thread-pool hop per call, which is negligible for interactive use.
- `async-trait` boxes each returned future; this is acceptable for I/O-bound database calls. We can drop the crate if dyn-compatible native async traits become available.
- Raising the MSRV later requires a deliberate, documented change; contributors on older toolchains must upgrade.
