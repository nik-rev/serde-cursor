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

#[test]
#[cfg(feature = "serde_with")]
fn serde_as_integration_full_roundtrip() {
    use serde::Deserialize;
    use serde::Serialize;
    use serde_with::serde_as;

    #[serde_as]
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct FullProject {
        #[serde(rename = "workspace")]
        #[serde_as(as = "Cursor!(package.version)")]
        version: String,
        name: String,
    }

    let input = json!({
        "name": "my-crate",
        "workspace": {
            "package": { "version": "1.2.3" },
        }
    });

    let project: FullProject = serde_json::from_value(input.clone()).unwrap();

    assert_eq!(project.version, "1.2.3");

    let output = serde_json::to_value(&project).unwrap();

    assert_eq!(input, output);
}

#[test]
fn borrowed_str_manual() {
    let json_str = r#"{"project":{"name":"serde-cursor"}}"#;
    let expected = "serde-cursor";

    type MyCursor<'a> = Cursor!(project.name: &'a str);

    let cursor: MyCursor = serde_json::from_str(json_str).unwrap();

    assert_eq!(*cursor, expected);

    let serialized = serde_json::to_string(&cursor).unwrap();
    assert_eq!(serialized, json_str);
}

#[test]
fn wildcard_with_missing_fields() {
    // some objects have 'val', one has 'other', one is empty
    let json = json!([
        { "val": 1 },
        { "other": 99 },
        { "val": 2 },
        {}
    ]);

    // type Foo<T> = ::serde_cursor::CursorPath<
    //     ::serde_cursor::FieldName<
    //         (
    //             ::serde_cursor::StrLen<7>,
    //             (
    //                 ::serde_cursor::C1<'p'>,
    //                 ::serde_cursor::C1<'a'>,
    //                 ::serde_cursor::C1<'c'>,
    //                 ::serde_cursor::C1<'k'>,
    //                 ::serde_cursor::C1<'a'>,
    //                 (::serde_cursor::C1<'g'>, ::serde_cursor::C1<'e'>),
    //             ),
    //         ),
    //         {
    //             [""];
    //             false
    //         },
    //     >,
    //     T,
    // >;

    // type Bar = ::serde_cursor::Cursor<
    //     String,
    //     Foo<
    //         ::serde_cursor::CursorPath<
    //             ::serde_cursor::Wildcard,
    //             ::serde_cursor::CursorPath<
    //                 ::serde_cursor::FieldName<
    //                     (
    //                         ::serde_cursor::StrLen<12>,
    //                         (
    //                             ::serde_cursor::C1<'d'>,
    //                             ::serde_cursor::C1<'e'>,
    //                             ::serde_cursor::C1<'p'>,
    //                             ::serde_cursor::C1<'e'>,
    //                             (
    //                                 ::serde_cursor::C1<'n'>,
    //                                 ::serde_cursor::C1<'d'>,
    //                                 ::serde_cursor::C1<'e'>,
    //                                 ::serde_cursor::C1<'n'>,
    //                                 ::serde_cursor::C1<'c'>,
    //                                 ::serde_cursor::C1<'i'>,
    //                             ),
    //                             (::serde_cursor::C1<'e'>, ::serde_cursor::C1<'s'>),
    //                         ),
    //                     ),
    //                     {
    //                         [""];
    //                         false
    //                     },
    //                 >,
    //                 ::serde_cursor::CursorPath<
    //                     ::serde_cursor::Index<0>,
    //                     ::serde_cursor::CursorPathEnd,
    //                 >,
    //             >,
    //         >,
    //     >,
    // >;
    // type Foxo<T> = CursorPath!(*.dependencies + T);

    // type Barxo = Cursor!(package.$Foxo.0: String);

    let cursor: Vec<Option<i32>> = serde_json::from_value::<Cursor!(*.val)>(json.clone())
        .unwrap()
        .0;
    assert_eq!(*cursor, vec![Some(1), None, Some(2), None]);

    let output = serde_json::to_value(&cursor).unwrap();

    assert_eq!(output, json!([1, null, 2, null]));
}

#[test]
fn type_mismatch_error() {
    let json = json!({ "a": { "not_an_array": 42 } });

    // path expects an array at index 0, but finds a map
    let result = serde_json::from_value::<Cursor!(a.0: i32)>(json);

    assert!(result.is_err());
}

#[test]
fn matrix_wildcards() {
    let json = json!({
        "matrix": [
            [{"v": 1}, {"v": 2}],
            [{"v": 3}]
        ]
    });

    type MatrixQuery = Cursor!(matrix.*.*.v: Vec<Vec<i32>>);

    let cursor: MatrixQuery = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(*cursor, vec![vec![1, 2], vec![3]]);

    let output = serde_json::to_value(&cursor).unwrap();
    assert_eq!(output, json);
}

#[test]
#[allow(clippy::type_complexity)]
fn empty_json_behaviors() {
    // path exists but value is null
    let json = json!({"a": null});
    let cursor: Cursor!(a: Option<i32>) = serde_json::from_value(json).unwrap();
    assert_eq!(*cursor, None);

    // index into empty array
    let json_empty_arr = json!({"arr": []});
    let cursor_idx: Result<Cursor!(arr.5: i32), _> = serde_json::from_value(json_empty_arr);
    assert!(cursor_idx.is_err(), "Indexing out of bounds should error");
}
