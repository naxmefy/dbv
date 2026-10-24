# 0003. Generic lazy object tree

Date: 2026-10-23

Status: Accepted

## Context

Databases organise their objects differently. SQLite has tables, views, indexes and triggers; Postgres adds databases, schemas and functions; MySQL has its own shape; document stores have collections. The explorer panel must show all of them without a hard-coded hierarchy, and large servers can contain far too many objects to load up front.

## Decision

Databases are exposed as a lazily loaded tree addressed by `ObjectPath`, a list of path segments such as `["tables", "orders", "columns"]`.

- `Session::children(path)` returns exactly one level below `path` as `Object { name, kind, has_children }`.
- `ObjectKind` is a closed set of common kinds (category, database, schema, table, view, index, trigger, column, foreign key, procedure) plus `Other(String)` for anything else.
- The driver defines which paths exist and what they contain; the root path `[]` is the entry point. The UI only walks the tree: Enter descends, Esc ascends, and the path is shown as a breadcrumb.
- The UI never assumes a particular hierarchy. For example, SQLite's root holds the categories `tables`, `views`, `indexes` and `triggers`, while a Postgres driver can add databases and schemas above them.
- Selecting an object that has data (a table or view) asks the driver for `preview`, which returns a page of rows.

## Consequences

- New databases, including non-relational ones, plug in by describing their own tree; no UI change is needed.
- Only the visible level is fetched, so big catalogs stay responsive.
- The UI cannot offer hierarchy-specific features (such as a schema picker) without going through `ObjectKind`; this is accepted for now.
- Each driver must document its path scheme, and path segments must be stable enough to use as keys.
