//! SQLite driver for dbv.

mod session;
mod tree;
mod values;

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use dbv_core::{ConnectionConfig, Driver, Error, Origin, Result, Session, StatementLog};
use rusqlite::{Connection, OpenFlags};

use session::{SqliteSession, logged};

/// `execute` returns at most this many rows and sets `truncated` beyond it.
pub const EXECUTE_ROW_CAP: usize = 10_000;

pub struct SqliteDriver;

/// Quote an identifier for use in SQL: `my "table"` → `"my ""table"""`.
pub fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

#[async_trait]
impl Driver for SqliteDriver {
    fn kind(&self) -> &'static str {
        "sqlite"
    }

    async fn connect(
        &self,
        cfg: &ConnectionConfig,
        log: Arc<dyn StatementLog>,
    ) -> Result<Box<dyn Session>> {
        let path = match cfg.options.get("path") {
            Some(toml::Value::String(p)) if !p.is_empty() => expand_home(p),
            _ => {
                return Err(Error::InvalidOption {
                    key: "path".into(),
                    message: "a path to the SQLite file is required".into(),
                });
            }
        };
        let conn = tokio::task::spawn_blocking(move || {
            // No SQLITE_OPEN_CREATE: a typo in the path must not create an empty database.
            let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_URI
                | OpenFlags::SQLITE_OPEN_NO_MUTEX;
            let conn = Connection::open_with_flags(&path, flags)
                .map_err(|e| Error::Database(format!("{}: {e}", path)))?;
            let sql = "SELECT count(*) FROM sqlite_schema";
            logged(
                log.as_ref(),
                sql,
                Origin::Internal,
                || conn.query_row(sql, [], |r| r.get::<_, i64>(0)).map(|_| 1),
                |_| dbv_core::Outcome::Rows(1),
            )?;
            Ok::<_, Error>(SqliteSession::new(Arc::new(Mutex::new(conn)), log))
        })
        .await
        .map_err(|e| Error::Database(e.to_string()))??;
        Ok(Box::new(conn))
    }
}

fn expand_home(path: &str) -> String {
    match (path.strip_prefix('~'), directories::BaseDirs::new()) {
        (Some(rest), Some(dirs)) if rest.is_empty() || rest.starts_with(['/', '\\']) => {
            format!("{}{rest}", dirs.home_dir().display())
        }
        _ => path.to_owned(),
    }
}
