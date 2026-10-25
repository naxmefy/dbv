use std::path::Path;
use std::sync::{Arc, Mutex};

use dbv_core::{
    ConnectionConfig, Driver, Error, LogEntry, ObjectPath, Origin, Outcome, Page, QueryResult,
    Session, StatementLog, Value,
};
use dbv_sqlite::SqliteDriver;
use tempfile::TempDir;

#[derive(Default)]
struct VecLog(Mutex<Vec<LogEntry>>);

impl StatementLog for VecLog {
    fn record(&self, entry: LogEntry) {
        self.0.lock().unwrap().push(entry);
    }
}

fn config(path: &Path) -> ConnectionConfig {
    let mut options = toml::Table::new();
    options.insert("path".into(), path.to_str().unwrap().into());
    ConnectionConfig {
        id: "test".into(),
        name: "test".into(),
        driver: "sqlite".into(),
        options,
    }
}

fn fixture() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let conn = rusqlite::Connection::open(dir.path().join("shop.db")).unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE orders (id INTEGER PRIMARY KEY, customer TEXT, total REAL, note BLOB);
        CREATE TABLE items (id INTEGER PRIMARY KEY, order_id INTEGER REFERENCES orders(id));
        CREATE TABLE "weird ""name" ("a b" TEXT, "ü" INTEGER);
        CREATE VIEW big_orders AS SELECT * FROM orders WHERE total > 100;
        CREATE INDEX orders_customer ON orders(customer);
        CREATE TRIGGER orders_touch AFTER UPDATE ON orders BEGIN SELECT 1; END;
        INSERT INTO "weird ""name" VALUES ('x', 1);
        "#,
    )
    .unwrap();
    dir
}

async fn open(dir: &TempDir) -> (Box<dyn Session>, Arc<VecLog>) {
    let log = Arc::new(VecLog::default());
    let session = SqliteDriver
        .connect(&config(&dir.path().join("shop.db")), log.clone())
        .await
        .unwrap();
    (session, log)
}

fn names(objects: &[dbv_core::Object]) -> Vec<&str> {
    objects.iter().map(|o| o.name.as_str()).collect()
}

fn path(segs: &[&str]) -> ObjectPath {
    segs.iter().fold(ObjectPath::root(), |p, s| p.child(s))
}

fn rows(result: QueryResult) -> (Vec<Vec<Value>>, bool) {
    match result {
        QueryResult::Rows {
            rows, truncated, ..
        } => (rows, truncated),
        other => panic!("expected rows, got {other:?}"),
    }
}

#[tokio::test]
async fn root_children_are_categories() {
    let dir = fixture();
    let (mut s, _) = open(&dir).await;
    let objs = s.children(&ObjectPath::root()).await.unwrap();
    assert_eq!(names(&objs), ["tables", "views", "indexes", "triggers"]);
    assert!(objs.iter().all(|o| o.has_children));
}

#[tokio::test]
async fn tables_listed_sorted_without_internals() {
    let dir = fixture();
    let (mut s, _) = open(&dir).await;
    // AUTOINCREMENT creates the internal sqlite_sequence table.
    s.execute("CREATE TABLE seq (id INTEGER PRIMARY KEY AUTOINCREMENT)")
        .await
        .unwrap();
    let objs = s.children(&path(&["tables"])).await.unwrap();
    assert_eq!(names(&objs), ["items", "orders", "seq", "weird \"name"]);
    assert_eq!(
        names(&s.children(&path(&["views"])).await.unwrap()),
        ["big_orders"]
    );
    let indexes = s.children(&path(&["indexes"])).await.unwrap();
    assert_eq!(names(&indexes), ["orders_customer"]);
    assert!(!indexes[0].has_children);
    assert_eq!(
        names(&s.children(&path(&["triggers"])).await.unwrap()),
        ["orders_touch"]
    );
}

#[tokio::test]
async fn table_children_and_columns() {
    let dir = fixture();
    let (mut s, _) = open(&dir).await;
    let cats = s.children(&path(&["tables", "orders"])).await.unwrap();
    assert_eq!(names(&cats), ["columns", "indexes", "foreign_keys"]);
    let cols = s
        .children(&path(&["tables", "orders", "columns"]))
        .await
        .unwrap();
    assert_eq!(names(&cols), ["id", "customer", "total", "note"]);
    let idx = s
        .children(&path(&["tables", "orders", "indexes"]))
        .await
        .unwrap();
    assert_eq!(names(&idx), ["orders_customer"]);
    let view = s.children(&path(&["views", "big_orders"])).await.unwrap();
    assert_eq!(names(&view), ["columns"]);
    assert!(matches!(
        s.children(&path(&["tables", "nope"])).await,
        Err(Error::NotFound(_))
    ));
    assert!(matches!(
        s.children(&path(&["bogus"])).await,
        Err(Error::NotFound(_))
    ));
}

#[tokio::test]
async fn foreign_keys_listed() {
    let dir = fixture();
    let (mut s, _) = open(&dir).await;
    let fks = s
        .children(&path(&["tables", "items", "foreign_keys"]))
        .await
        .unwrap();
    assert_eq!(names(&fks), ["order_id → orders.id"]);
}

#[tokio::test]
async fn preview_pages() {
    let dir = fixture();
    let (mut s, _) = open(&dir).await;
    s.execute(
        "INSERT INTO orders (customer, total) \
         WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 250) \
         SELECT 'c' || i, i FROM n",
    )
    .await
    .unwrap();
    let page = Page {
        offset: 200,
        limit: 100,
    };
    let (r, truncated) = rows(s.preview(&path(&["tables", "orders"]), page).await.unwrap());
    assert_eq!(r.len(), 50);
    assert!(!truncated);
    assert!(matches!(
        s.preview(&path(&["indexes", "orders_customer"]), page)
            .await,
        Err(Error::Unsupported(_))
    ));
}

#[tokio::test]
async fn preview_quotes_weird_names() {
    let dir = fixture();
    let (mut s, _) = open(&dir).await;
    let page = Page {
        offset: 0,
        limit: 100,
    };
    let (r, _) = rows(
        s.preview(&path(&["tables", "weird \"name"]), page)
            .await
            .unwrap(),
    );
    assert_eq!(r, [[Value::Text("x".into()), Value::Int(1)]]);
}

#[tokio::test]
async fn execute_select_and_values() {
    let dir = fixture();
    let (mut s, _) = open(&dir).await;
    s.execute("INSERT INTO orders VALUES (1, '张伟', 12.5, x'010203')")
        .await
        .unwrap();
    s.execute("INSERT INTO orders VALUES (2, NULL, NULL, NULL)")
        .await
        .unwrap();
    let QueryResult::Rows { columns, rows, .. } = s
        .execute("SELECT id, customer, total, note FROM orders ORDER BY id")
        .await
        .unwrap()
    else {
        panic!("expected rows")
    };
    assert_eq!(columns[0].type_name, "INTEGER");
    assert_eq!(columns[1].name, "customer");
    assert_eq!(
        rows[0],
        [
            Value::Int(1),
            Value::Text("张伟".into()),
            Value::Float(12.5),
            Value::Bytes(vec![1, 2, 3])
        ]
    );
    assert_eq!(rows[1][1..], [Value::Null, Value::Null, Value::Null]);
    let QueryResult::Rows { columns, .. } = s.execute("SELECT 1 + 1 AS two").await.unwrap() else {
        panic!("expected rows")
    };
    assert_eq!(columns[0].type_name, "");
}

#[tokio::test]
async fn execute_update_returns_affected() {
    let dir = fixture();
    let (mut s, _) = open(&dir).await;
    s.execute("INSERT INTO orders (customer) VALUES ('a'), ('b'), ('c')")
        .await
        .unwrap();
    let r = s
        .execute("UPDATE orders SET total = 1 WHERE customer <> 'a'")
        .await;
    assert_eq!(r.unwrap(), QueryResult::Affected(2));
}

#[tokio::test]
async fn execute_caps_rows() {
    let dir = fixture();
    let (mut s, _) = open(&dir).await;
    let (r, truncated) = rows(
        s.execute(
            "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 10001) \
             SELECT i FROM n",
        )
        .await
        .unwrap(),
    );
    assert_eq!(r.len(), 10_000);
    assert!(truncated);
}

#[tokio::test]
async fn execute_rejects_multiple_statements() {
    let dir = fixture();
    let (mut s, _) = open(&dir).await;
    assert!(matches!(
        s.execute("SELECT 1; SELECT 2;").await,
        Err(Error::Database(_))
    ));
    // A trailing semicolon or comment is still a single statement.
    assert!(s.execute("SELECT 1; -- done\n").await.is_ok());
}

#[tokio::test]
async fn execute_syntax_error_is_err() {
    let dir = fixture();
    let (mut s, log) = open(&dir).await;
    assert!(matches!(s.execute("SELEC").await, Err(Error::Database(_))));
    let last = log.0.lock().unwrap().last().cloned().unwrap();
    assert_eq!(last.statement, "SELEC");
    assert_eq!(last.origin, Origin::User);
    assert!(matches!(last.outcome, Outcome::Error(_)));
}

#[tokio::test]
async fn connect_missing_file_does_not_create() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("missing.db");
    let r = SqliteDriver
        .connect(&config(&file), Arc::new(VecLog::default()))
        .await;
    assert!(matches!(r, Err(Error::Database(_))));
    assert!(!file.exists());
}

#[tokio::test]
async fn connect_non_sqlite_file_fails() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("notes.txt");
    std::fs::write(&file, "this is not a database, just some text").unwrap();
    let r = SqliteDriver
        .connect(&config(&file), Arc::new(VecLog::default()))
        .await;
    assert!(matches!(r, Err(Error::Database(_))));
}

#[tokio::test]
async fn connect_missing_path_option() {
    let mut cfg = config(Path::new("x"));
    cfg.options.clear();
    let r = SqliteDriver
        .connect(&cfg, Arc::new(VecLog::default()))
        .await;
    assert!(matches!(r, Err(Error::InvalidOption { key, .. }) if key == "path"));
}

#[tokio::test]
async fn every_statement_is_logged() {
    let dir = fixture();
    let (mut s, log) = open(&dir).await;
    let before = log.0.lock().unwrap().len();
    assert!(before >= 1, "connect validation is logged");
    s.children(&path(&["tables"])).await.unwrap();
    s.preview(
        &path(&["tables", "orders"]),
        Page {
            offset: 0,
            limit: 10,
        },
    )
    .await
    .unwrap();
    s.execute("SELECT 1").await.unwrap();
    let entries = log.0.lock().unwrap();
    let new = &entries[before..];
    assert_eq!(new.len(), 3);
    assert_eq!(new[0].origin, Origin::Internal);
    assert_eq!(new[1].origin, Origin::Internal);
    assert_eq!(new[2].origin, Origin::User);
    assert_eq!(new[2].outcome, Outcome::Rows(1));
    assert_eq!(
        dbv_sqlite::quote_ident("weird \"name"),
        "\"weird \"\"name\""
    );
}
