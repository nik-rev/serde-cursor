#![allow(clippy::type_complexity)]

use serde_cursor::Cursor;
use serde_json::from_value;
use serde_json::json;

#[test]
fn test_error_missing_field() {
    let data = json!({ "author": { "id": 1 } });
    let result = from_value::<Cursor!(author.name: String)>(data);
    let err = result.unwrap_err().to_string();
    // Fixed spacing: ".author.name: ..."
    assert_eq!(err, ".author.name: missing field 'name'");
}

#[test]
fn test_error_type_mismatch() {
    let data = json!({ "author": { "id": "not-a-number" } });
    let result = from_value::<Cursor!(author.id: i32)>(data);
    let err = result.unwrap_err().to_string();
    assert!(err.starts_with(".author.id: invalid type: string"));
}

#[test]
fn test_error_wildcard_index_tracking() {
    let data = json!([
        { "name": "alice" },
        { "typo": "bob" }
    ]);

    let result = from_value::<Cursor!(*.name: Vec<String>)>(data);
    let err = result.unwrap_err().to_string();

    assert_eq!(err, "[1].name: missing field 'name'");
}

#[test]
fn test_error_deep_wildcard_mismatch() {
    let data = json!({
        "org": {
            "users": [
                { "id": 1 },
                { "id": "invalid" }
            ]
        }
    });
    let result = from_value::<Cursor!(org.users.*.id: Vec<i32>)>(data);
    let err = result.unwrap_err().to_string();

    assert_eq!(
        err,
        ".org.users[1].id: invalid type: string \"invalid\", expected i32"
    );
}

#[test]
fn type_mismatch_error() {
    let json = json!({ "a": { "not_an_array": 42 } });

    // path expects an array at index 0, but finds a map
    let result = serde_json::from_value::<Cursor!(a.0: i32)>(json);

    assert!(result.is_err());
}

#[test]
fn empty_json_behaviors() {
    // path exists but value is null
    let json = json!({"a": null});
    let cursor: Cursor!(a: Option<i32>) = serde_json::from_value(json).unwrap();
    assert_eq!(*cursor, None);

    // index into empty array
    let json_empty_arr = json!({"arr": []});
    let cursor_idx: Result<Cursor!(arr.5: i32), _> = serde_json::from_value(json_empty_arr);
    assert!(cursor_idx.is_err());
}
