use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, PartialEq)]
pub enum Origin {
    /// Issued by dbv itself (tree loading, previews, validation).
    Internal,
    /// Typed by the user.
    User,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Rows(u64),
    Affected(u64),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub statement: String,
    pub origin: Origin,
    pub started_at: SystemTime,
    pub duration: Duration,
    pub outcome: Outcome,
}

/// Receives every statement a driver executes.
pub trait StatementLog: Send + Sync {
    fn record(&self, entry: LogEntry);
}
