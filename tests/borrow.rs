use serde_cursor::Cursor;

#[test]
fn borrowed_str() {
    let json_str = r#"{"project":{"name":"serde-cursor"}}"#;
    let expected = "serde-cursor";

    type MyCursor<'a> = Cursor!(project.name: &'a str);

    let cursor: MyCursor = serde_json::from_str(json_str).unwrap();

    assert_eq!(*cursor, expected);

    let serialized = serde_json::to_string(&cursor).unwrap();
    assert_eq!(serialized, json_str);
}
