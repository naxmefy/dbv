use crate::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub name: String,
    /// Declared type, or empty if the database does not report one.
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryResult {
    Rows {
        columns: Vec<Column>,
        rows: Vec<Vec<Value>>,
        /// More rows existed than the driver returned.
        truncated: bool,
    },
    Affected(u64),
}
