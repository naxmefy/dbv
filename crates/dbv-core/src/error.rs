#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("invalid option `{key}`: {message}")]
    InvalidOption { key: String, message: String },
    #[error("not found: {0}")]
    NotFound(String),
    #[error("not supported: {0}")]
    Unsupported(String),
    #[error("{0}")]
    Database(String),
    #[error("connection is closed")]
    Closed,
}

pub type Result<T> = std::result::Result<T, Error>;
