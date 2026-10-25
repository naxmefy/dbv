use std::borrow::Cow;

/// A single cell value, independent of the database it came from.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Bytes(Vec<u8>),
    /// Driver-specific value already rendered as text.
    Other(String),
}

impl Value {
    pub fn display(&self) -> Cow<'_, str> {
        match self {
            Value::Null => Cow::Borrowed("NULL"),
            Value::Bool(b) => Cow::Owned(b.to_string()),
            Value::Int(i) => Cow::Owned(i.to_string()),
            Value::Float(f) => Cow::Owned(f.to_string()),
            Value::Text(s) | Value::Other(s) => Cow::Borrowed(s),
            Value::Bytes(b) => Cow::Owned(format!("<{} bytes>", b.len())),
        }
    }
}
