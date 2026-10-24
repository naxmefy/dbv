use dbv_core::{ConnectionConfig, ObjectPath, Value};

#[test]
fn object_path_navigation() {
    let p = ObjectPath::root().child("tables").child("orders");
    assert_eq!(p.segments(), ["tables", "orders"]);
    assert_eq!(p.parent().unwrap().segments(), ["tables"]);
    assert!(ObjectPath::root().parent().is_none());
    assert!(ObjectPath::root().is_root());
}

#[test]
fn value_display() {
    assert_eq!(Value::Null.display(), "NULL");
    assert_eq!(Value::Bytes(vec![0; 3]).display(), "<3 bytes>");
    assert_eq!(Value::Text("张伟".into()).display(), "张伟");
}

#[test]
fn connection_config_roundtrip_toml() {
    let mut options = toml::Table::new();
    options.insert("path".into(), "~/data/shop.db".into());
    let cfg = ConnectionConfig {
        id: "01J".into(),
        name: "shop".into(),
        driver: "sqlite".into(),
        options,
    };
    let text = toml::to_string(&cfg).unwrap();
    let back: ConnectionConfig = toml::from_str(&text).unwrap();
    assert_eq!(back.options["path"].as_str(), Some("~/data/shop.db"));
    assert_eq!(back.name, "shop");

    let no_options: ConnectionConfig =
        toml::from_str("id = \"1\"\nname = \"x\"\ndriver = \"sqlite\"\n").unwrap();
    assert!(no_options.options.is_empty());
}
