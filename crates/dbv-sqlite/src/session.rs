use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime};

use async_trait::async_trait;
use dbv_core::{
    Column, Error, LogEntry, Object, ObjectPath, Origin, Outcome, Page, QueryResult, Result,
    Session, StatementLog,
};
use rusqlite::Connection;

use crate::{EXECUTE_ROW_CAP, quote_ident, tree, values};

pub(crate) struct SqliteSession {
    conn: Arc<Mutex<Connection>>,
    log: Arc<dyn StatementLog>,
}

impl SqliteSession {
    pub(crate) fn new(conn: Arc<Mutex<Connection>>, log: Arc<dyn StatementLog>) -> Self {
        Self { conn, log }
    }

    /// Run `f` on the connection in a blocking task.
    async fn with_conn<T: Send + 'static>(
        &self,
        f: impl FnOnce(&Connection, &dyn StatementLog) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        let conn = self.conn.clone();
        let log = self.log.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|_| Error::Database("connection lock poisoned".into()))?;
            f(&conn, log.as_ref())
        })
        .await
        .map_err(|e| Error::Database(e.to_string()))?
    }
}

/// Run one SQL call and record exactly one log entry for it.
pub(crate) fn logged<T>(
    log: &dyn StatementLog,
    sql: &str,
    origin: Origin,
    f: impl FnOnce() -> rusqlite::Result<T>,
    outcome: impl Fn(&T) -> Outcome,
) -> Result<T> {
    let started_at = SystemTime::now();
    let start = Instant::now();
    let result = f().map_err(db_error);
    log.record(LogEntry {
        statement: sql.to_owned(),
        origin,
        started_at,
        duration: start.elapsed(),
        outcome: match &result {
            Ok(v) => outcome(v),
            Err(e) => Outcome::Error(e.to_string()),
        },
    });
    result
}

fn db_error(e: rusqlite::Error) -> Error {
    match e {
        rusqlite::Error::MultipleStatement => {
            Error::Database("multiple statements are not supported".into())
        }
        e => Error::Database(e.to_string()),
    }
}

fn rows_outcome(r: &QueryResult) -> Outcome {
    match r {
        QueryResult::Rows { rows, .. } => Outcome::Rows(rows.len() as u64),
        QueryResult::Affected(n) => Outcome::Affected(*n),
    }
}

fn run(conn: &Connection, sql: &str, cap: usize) -> rusqlite::Result<QueryResult> {
    let mut stmt = conn.prepare(sql)?;
    if stmt.column_count() == 0 {
        return Ok(QueryResult::Affected(stmt.execute([])? as u64));
    }
    let columns: Vec<Column> = stmt
        .columns()
        .iter()
        .map(|c| Column {
            name: c.name().to_owned(),
            type_name: c.decl_type().unwrap_or("").to_owned(),
        })
        .collect();
    let mut rows = Vec::new();
    let mut truncated = false;
    let mut cursor = stmt.query([])?;
    while let Some(row) = cursor.next()? {
        if rows.len() == cap {
            truncated = true;
            break;
        }
        rows.push(values::row(row, columns.len())?);
    }
    Ok(QueryResult::Rows {
        columns,
        rows,
        truncated,
    })
}

#[async_trait]
impl Session for SqliteSession {
    async fn children(&mut self, path: &ObjectPath) -> Result<Vec<Object>> {
        let path = path.clone();
        self.with_conn(move |conn, log| tree::children(conn, log, &path))
            .await
    }

    async fn execute(&mut self, query: &str) -> Result<QueryResult> {
        let sql = query.to_owned();
        self.with_conn(move |conn, log| {
            logged(
                log,
                &sql,
                Origin::User,
                || run(conn, &sql, EXECUTE_ROW_CAP),
                rows_outcome,
            )
        })
        .await
    }

    async fn preview(&mut self, path: &ObjectPath, page: Page) -> Result<QueryResult> {
        let name = match path.segments() {
            [cat, name] if cat == "tables" || cat == "views" => name.clone(),
            _ => {
                return Err(Error::Unsupported(format!(
                    "preview of {}",
                    path.segments().join("/")
                )));
            }
        };
        // LIMIT/OFFSET are integers, so inlining them keeps the logged statement readable.
        let sql = format!(
            "SELECT * FROM {} LIMIT {} OFFSET {}",
            quote_ident(&name),
            page.limit,
            page.offset
        );
        self.with_conn(move |conn, log| {
            logged(
                log,
                &sql,
                Origin::Internal,
                || run(conn, &sql, usize::MAX),
                rows_outcome,
            )
        })
        .await
    }

    async fn close(self: Box<Self>) -> Result<()> {
        Ok(())
    }
}
