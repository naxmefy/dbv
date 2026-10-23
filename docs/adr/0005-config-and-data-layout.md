# 0005. Configuration and data layout

Date: 2026-10-23

Status: Accepted

## Context

dbv stores user settings, saved connections, per-connection query history and saved scripts. Users will want to edit some of it by hand, version some of it, and never lose any of it to a crash or a typo. Paths must follow each operating system's conventions.

## Decision

Locations are resolved with the `directories` crate and can be overridden with `--config <dir>` and `--data <dir>`.

**Config directory** (user-edited, small):

- `connections.toml`: a list of `[[connection]]` tables with a stable `id` (ULID), `name`, `driver`, and a driver-specific `[connection.options]` table validated by the driver.
- `config.toml`: optional settings such as keybinding overrides.

**Data directory** (produced by dbv, per connection):

```
connections/<connection-id>/
  history.jsonl        one JSON object per executed query, append-only
  scripts/*.sql        plain SQL files; file name is the script name
```

The folder is named after the connection ID, not its name, so renaming a connection orphans nothing. History stores the query and metadata only, never results, and is capped (default 500 entries per connection).

**Rules:**

- Writes are atomic: write a temporary file, then rename.
- A file that fails to parse produces an error that names the file and line; dbv never overwrites it.
- Unknown keybinding actions or conflicts produce a warning at startup; defaults stay active.
- Passwords are never stored in plaintext by default. M1 needs none (SQLite); how they are stored is decided in a separate ADR in M3.

## Consequences

- Scripts are ordinary files: editable in any editor, easy to version or sync.
- Reading and writing use simple, human-inspectable formats; no database is needed for dbv's own state.
- Rewriting `history.jsonl` on deletion and capping it are the only non-append operations, and both go through the atomic path.
- Deleting a connection must ask whether its data folder goes too.
