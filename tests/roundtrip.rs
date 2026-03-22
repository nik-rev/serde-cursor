#![allow(clippy::type_complexity)]

use serde_cursor::Cursor;
use serde_json::json;

/// `C` satisfies any type returned by `Cursor!` macro
#[track_caller]
fn assert_roundtrip<T, C>(input: serde_json::Value, expected_inner: T)
where
    T: PartialEq + std::fmt::Debug,
    C: serde::de::DeserializeOwned + serde::Serialize + std::ops::Deref<Target = T>,
{
    // JSON -> Cursor
    let cursor: C = serde_json::from_value(input.clone()).unwrap();

    assert_eq!(*cursor, expected_inner);

    // Cursor -> JSON
    let output = serde_json::to_value(&cursor).unwrap();

    assert_eq!(input, output);
}

#[test]
fn deep_field_path() {
    let json = json!({
        "a": { "b": { "c": 100 } }
    });

    assert_roundtrip::<i32, Cursor!(a.b.c)>(json, 100);
}

#[test]
fn array_index_path() {
    // indices create null-padding for preceding elements during serialization
    let json = json!({
        "arr": [null, null, "found me"]
    });

    assert_roundtrip::<String, Cursor!(arr.2)>(json, "found me".to_string());
}

#[test]
fn crab() {
    let json = json!({
        "🦀": "crab"
    });

    assert_roundtrip::<String, Cursor!("🦀")>(json, "crab".to_string());
}

#[test]
fn with_dashes() {
    let json = json!({
        "--dev-dependencies": {
            "--yes": true
        }
    });

    assert_roundtrip::<bool, Cursor!(--dev-dependencies.--yes)>(json, true);
}

#[test]
fn wildcard_collection() {
    let json = json!([
        { "val": 10 },
        { "val": 20 }
    ]);

    assert_roundtrip::<Vec<i32>, Cursor!(*.val)>(json, vec![10, 20]);
}

#[test]
fn mixed_nested_path() {
    let json = json!({
        "users": [
            null,
            { "meta": { "id": "uuid-1" } }
        ]
    });

    assert_roundtrip::<String, Cursor!(users.1.meta.id)>(json, "uuid-1".to_string());
}

#[test]
fn nested_wildcards() {
    let json = json!({
        "groups": [
            { "members": [{ "name": "A" }, { "name": "B" }] },
            { "members": [{ "name": "C" }] }
        ]
    });

    let expected = vec![
        vec!["A".to_string(), "B".to_string()],
        vec!["C".to_string()],
    ];

    assert_roundtrip::<Vec<Vec<String>>, Cursor!(groups.*.members.*.name)>(json, expected);
}

#[test]
fn complex_wildcard_objects() {
    let json = json!({
        "data": [
            { "info": { "code": 1 } },
            { "info": { "code": 2 } }
        ]
    });
    assert_roundtrip::<Vec<i32>, Cursor!(data.*.info.code)>(json, vec![1, 2]);
}
