//! Tests for `$Interpolation` syntax.

use serde_cursor::Cursor;
use serde_cursor::CursorPath;
use serde_json::json;

#[test]
fn at_end() -> evil::Result {
    let data = json!({ "properties": { "timeseries": [1.0] } });

    type Details<T> = CursorPath!(timeseries + T);

    let timeseries: Vec<f64> = serde_json::from_value::<Cursor!(properties.$Details)>(data)?.0;

    assert_eq!(timeseries, vec![1.0]);

    evil::Ok(())
}

#[test]
fn at_start() -> evil::Result {
    let data = json!({ "properties": { "timeseries": [1.0] } });

    type Details<T> = CursorPath!(properties + T);

    let timeseries: Vec<f64> = serde_json::from_value::<Cursor!($Details.timeseries)>(data)?.0;

    assert_eq!(timeseries, vec![1.0]);

    evil::Ok(())
}

#[test]
fn entire() -> evil::Result {
    let data = json!({ "properties": { "timeseries": [1.0] } });

    type Details<T> = CursorPath!(properties.timeseries + T);

    let timeseries: Vec<f64> = serde_json::from_value::<Cursor!($Details)>(data)?.0;

    assert_eq!(timeseries, vec![1.0]);

    evil::Ok(())
}

#[test]
fn in_middle() -> evil::Result {
    let data = json!({ "properties": { "timeseries": { "data": [1.0] } } });

    type Details<T> = CursorPath!(timeseries + T);

    let timeseries: Vec<f64> = serde_json::from_value::<Cursor!(properties.$Details.data)>(data)?.0;

    assert_eq!(timeseries, vec![1.0]);

    evil::Ok(())
}

#[test]
fn with_wildcard() -> evil::Result {
    let make_weather = |pressure: f64, humidity: f64, temp: f64| {
        json!({
            "properties": {
                "timeseries": [{
                    "data": {
                        "instant": {
                            "details": {
                                "air_pressure_at_sea_level": pressure,
                                "relative_humidity": humidity,
                                "air_temperature": temp
                            }
                        }
                    }
                }]
            }
        })
    };

    let france = json!({ "france": make_weather(1.0, 2.0, 3.0) });
    let japan = json!({ "japan": make_weather(4.0, 5.0, 6.0) });

    type Details<T> = CursorPath!(properties.timeseries.*.data.instant.details + T);

    let pressure: Vec<f64> =
        serde_json::from_value::<Cursor!(france.$Details.air_pressure_at_sea_level)>(france)?.0;
    let humidity: Vec<f64> =
        serde_json::from_value::<Cursor!(japan.$Details.relative_humidity)>(japan.clone())?.0;
    let temperature: Vec<f64> =
        serde_json::from_value::<Cursor!(japan.$Details.air_temperature)>(japan)?.0;

    assert_eq!(pressure, vec![1.0]);
    assert_eq!(humidity, vec![5.0]);
    assert_eq!(temperature, vec![6.0]);

    evil::Ok(())
}
