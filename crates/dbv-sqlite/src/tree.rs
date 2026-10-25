use dbv_core::{Error, Object, ObjectKind, ObjectPath, Origin, Outcome, Result, StatementLog};
use rusqlite::Connection;

use crate::session::logged;

fn obj(name: &str, kind: ObjectKind, has_children: bool) -> Object {
    Object {
        name: name.to_owned(),
        kind,
        has_children,
    }
}

fn categories(names: &[&str]) -> Vec<Object> {
    names
        .iter()
        .map(|n| obj(n, ObjectKind::Category, true))
        .collect()
}

/// Schema type and child kind for each top-level category.
fn category(name: &str) -> Option<(&'static str, ObjectKind, bool)> {
    match name {
        "tables" => Some(("table", ObjectKind::Table, true)),
        "views" => Some(("view", ObjectKind::View, true)),
        "indexes" => Some(("index", ObjectKind::Index, false)),
        "triggers" => Some(("trigger", ObjectKind::Trigger, false)),
        _ => None,
    }
}

fn names(
    conn: &Connection,
    log: &dyn StatementLog,
    sql: &str,
    arg: Option<&str>,
) -> Result<Vec<String>> {
    logged(
        log,
        sql,
        Origin::Internal,
        || {
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map(rusqlite::params_from_iter(arg), |r| r.get(0))?;
            rows.collect()
        },
        |v: &Vec<String>| Outcome::Rows(v.len() as u64),
    )
}

pub(crate) fn children(
    conn: &Connection,
    log: &dyn StatementLog,
    path: &ObjectPath,
) -> Result<Vec<Object>> {
    let not_found = || Error::NotFound(path.segments().join("/"));
    match path.segments() {
        [] => Ok(categories(&["tables", "views", "indexes", "triggers"])),
        [cat] => {
            let (ty, kind, has_children) = category(cat).ok_or_else(not_found)?;
            let sql = format!(
                "SELECT name FROM sqlite_schema WHERE type = '{ty}' \
                 AND name NOT LIKE 'sqlite\\_%' ESCAPE '\\' ORDER BY name"
            );
            Ok(names(conn, log, &sql, None)?
                .iter()
                .map(|n| obj(n, kind.clone(), has_children))
                .collect())
        }
        [cat, name] if cat == "tables" || cat == "views" => {
            let ty = if cat == "tables" { "table" } else { "view" };
            let sql = format!("SELECT name FROM sqlite_schema WHERE type = '{ty}' AND name = ?1");
            if names(conn, log, &sql, Some(name))?.is_empty() {
                return Err(not_found());
            }
            Ok(if cat == "tables" {
                categories(&["columns", "indexes", "foreign_keys"])
            } else {
                categories(&["columns"])
            })
        }
        [cat, name, sub] if cat == "tables" || (cat == "views" && sub == "columns") => {
            match sub.as_str() {
                "columns" => {
                    let sql = "SELECT name FROM pragma_table_info(?1) ORDER BY cid";
                    Ok(names(conn, log, sql, Some(name))?
                        .iter()
                        .map(|n| obj(n, ObjectKind::Column, false))
                        .collect())
                }
                "indexes" => {
                    let sql = "SELECT name FROM pragma_index_list(?1) ORDER BY name";
                    Ok(names(conn, log, sql, Some(name))?
                        .iter()
                        .map(|n| obj(n, ObjectKind::Index, false))
                        .collect())
                }
                "foreign_keys" => {
                    // `to` is NULL when the reference targets the primary key implicitly.
                    let sql = "SELECT \"from\" || ' → ' || \"table\" || \
                               coalesce('.' || \"to\", '') \
                               FROM pragma_foreign_key_list(?1) ORDER BY id, seq";
                    Ok(names(conn, log, sql, Some(name))?
                        .iter()
                        .map(|n| obj(n, ObjectKind::ForeignKey, false))
                        .collect())
                }
                _ => Err(not_found()),
            }
        }
        _ => Err(not_found()),
    }
}
