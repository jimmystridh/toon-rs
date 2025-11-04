#![cfg(feature = "json")]
use serde::Serialize;

#[derive(Serialize)]
struct Floats { a: f64, b: f64, c: f64 }

#[test]
fn serialize_non_finite_floats_as_strings() {
    let v = Floats { a: f64::NAN, b: f64::INFINITY, c: f64::NEG_INFINITY };
    let out = toon::ser::to_string(&v, &toon::Options::default()).unwrap();
    assert!(out.contains("a: \"NaN\""));
    assert!(out.contains("b: \"Infinity\""));
    assert!(out.contains("c: \"-Infinity\""));
}
