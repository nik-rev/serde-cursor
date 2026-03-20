use serde_cursor::Cursor;
use serde_json::json;

#[test]
fn lol() {
    let value = json!({
        "a": {
            "hello world": {
                "c": [false, false, true, false]
            }
        }
    });

    let lol = json!({
        "contentJsons": {
            "anchors": [
                {
                    "name": "sells",
                    "json": {
                        "elements": [49, 10, 80],
                    }
            },
            {
                "name": "prices",
                "json": {
                    "elements": [49, 89, 29],
                }
            },
            ]
        }
    });

    let x = serde_json::from_value::<Cursor!(contentJsons.anchors.*.name: Vec<String>)>(lol)
        .unwrap()
        .0;

    // let x = serde_json::from_value::<Cursor!(a."hello world".c.2: bool)>(value)
    //     .unwrap()
    //     .value;

    dbg!(x);
    panic!();
}
