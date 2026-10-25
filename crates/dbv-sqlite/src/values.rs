use dbv_core::Value;
use rusqlite::{Row, types::ValueRef};

pub(crate) fn row(row: &Row<'_>, len: usize) -> rusqlite::Result<Vec<Value>> {
    (0..len)
        .map(|i| {
            Ok(match row.get_ref(i)? {
                ValueRef::Null => Value::Null,
                ValueRef::Integer(v) => Value::Int(v),
                ValueRef::Real(v) => Value::Float(v),
                ValueRef::Text(v) => Value::Text(String::from_utf8_lossy(v).into_owned()),
                ValueRef::Blob(v) => Value::Bytes(v.to_vec()),
            })
        })
        .collect()
}
