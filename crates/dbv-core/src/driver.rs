use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{Object, ObjectPath, Page, QueryResult, Result, StatementLog};

/// A saved connection; `options` are driver-specific (e.g. `path` for SQLite).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub driver: String,
    #[serde(default)]
    pub options: toml::Table,
}

#[async_trait]
pub trait Driver: Send + Sync {
    fn kind(&self) -> &'static str;
    async fn connect(
        &self,
        cfg: &ConnectionConfig,
        log: Arc<dyn StatementLog>,
    ) -> Result<Box<dyn Session>>;
}

#[async_trait]
pub trait Session: Send {
    async fn children(&mut self, path: &ObjectPath) -> Result<Vec<Object>>;
    async fn execute(&mut self, query: &str) -> Result<QueryResult>;
    async fn preview(&mut self, path: &ObjectPath, page: Page) -> Result<QueryResult>;
    async fn close(self: Box<Self>) -> Result<()>;
}
