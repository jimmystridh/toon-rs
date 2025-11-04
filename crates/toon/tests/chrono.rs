#![cfg(feature = "chrono")]
use chrono::{DateTime, Utc, TimeZone};

#[derive(serde::Serialize)]
struct WithDate { ts: DateTime<Utc> }

#[test]
fn chrono_datetime_serializes_as_string() {
    let dt = Utc.with_ymd_and_hms(2024, 5, 1, 12, 34, 56).unwrap();
    let v = WithDate { ts: dt };
    let out = toon::ser::to_string(&v, &toon::Options::default()).unwrap();
    assert!(out.contains("ts: \"2024-05-01T12:34:56"));
}
