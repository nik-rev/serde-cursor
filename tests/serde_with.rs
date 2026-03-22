#![cfg(feature = "serde_with")]

use serde::Deserialize;
use serde::Serialize;
use serde_cursor::Cursor;
use serde_json::json;
use serde_with::serde_as;

#[test]
fn serde_as_integration_full_roundtrip() {
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
