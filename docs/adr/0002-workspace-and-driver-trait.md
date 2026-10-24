# 0002. Workspace layout and driver trait

Date: 2026-10-23

Status: Accepted

## Context

dbv starts with SQLite and is meant to grow to Postgres, MySQL, further SQL databases and eventually NoSQL stores. The UI must not depend on any concrete database, and each driver brings heavy dependencies that users should not have to compile unless they want them.

## Decision

We use a Cargo workspace with three crates:

- `dbv-core`: driver traits and shared types. No UI, no concrete database.
- `dbv-sqlite`: the SQLite driver (`rusqlite` with the `bundled` feature).
- `dbv`: the binary (configuration, history, scripts, keymap, i18n, TUI, later the CLI).

Every further driver is its own crate and is enabled in `dbv` through a Cargo feature, one per driver.

`dbv-core` defines the contract between the application and a database. Sketch (async methods are made dyn-compatible via `async-trait`, see ADR 0004):

```rust
pub trait Driver: Send + Sync {
    fn kind(&self) -> &'static str;
    async fn connect(&self, cfg: &ConnectionConfig, log: Arc<dyn StatementLog>)
        -> Result<Box<dyn Session>>;
}

pub trait Session: Send {
    async fn children(&mut self, path: &ObjectPath) -> Result<Vec<Object>>;
    async fn execute(&mut self, query: &str) -> Result<QueryResult>;
    async fn preview(&mut self, path: &ObjectPath, page: Page) -> Result<QueryResult>;
    async fn close(self: Box<Self>) -> Result<()>;
}

pub trait StatementLog: Send + Sync {
    fn record(&self, entry: LogEntry);
}
```

Rules: every statement a driver runs, including internal introspection queries, is reported to `StatementLog`. `preview` decides its own paging. Errors are `thiserror` enums in the core and drivers. Extensions (decimal values, document results, cancellation, DDL generation) are added only in the milestone that needs them.

Alternatives rejected:

- **Single crate.** Simpler at first, but module boundaries erode: UI code starts reaching into driver internals, and every user compiles every database dependency.
- **sqlx.** Its typed, driver-specific API fits applications with known schemas, not a viewer for arbitrary ones. Dynamic values are awkward, per-database introspection would still be ours to write, and NoSQL stores do not fit its model.

## Consequences

- Adding a database means adding a crate that implements the traits; the TUI does not change.
- Build time and binary size stay controllable through features.
- We own the value model and introspection per driver, which is more code than a generic library but fits a schema-agnostic viewer.
- The traits are deliberately small and will be revised as Postgres and MySQL expose gaps.
