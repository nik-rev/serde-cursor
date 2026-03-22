use serde_cursor::Cursor;
use serde_json::json;

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
fn wildcard_with_missing_fields() {
    // some objects have 'val', one has 'other', one is empty
    let json = json!([
        { "val": 1 },
        { "other": 99 },
        { "val": 2 },
        {}
    ]);

    let cursor: Vec<Option<i32>> = serde_json::from_value::<Cursor!(*.val)>(json.clone())
        .unwrap()
        .0;
    assert_eq!(*cursor, vec![Some(1), None, Some(2), None]);

    let output = serde_json::to_value(&cursor).unwrap();

    assert_eq!(output, json!([1, null, 2, null]));
}
