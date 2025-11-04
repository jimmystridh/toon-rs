#![cfg(all(feature = "serde", not(feature = "json")))]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Row { a: u32, b: String }

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Wrapper { rows: Vec<Row> }

#[test]
fn typed_roundtrip_alloc() -> Result<(), toon::Error> {
    let w = Wrapper { rows: vec![ Row { a: 1, b: "x".into() }, Row { a: 2, b: "y".into() } ] };
    let opts = toon::Options::default();
    let s = toon::ser::to_string_streaming(&w, &opts)?;
    let back: Wrapper = toon::de::from_str(&s, &opts)?;
    assert_eq!(w, back);
    Ok(())
}

#[test]
fn tabular_emission_alloc() -> Result<(), toon::Error> {
    let w = Wrapper { rows: vec![ Row { a: 1, b: "x".into() }, Row { a: 2, b: "y".into() } ] };
    let s = toon::ser::to_string_streaming(&w, &toon::Options::default())?;
    assert!(s.contains("@, a, b"));
    assert!(s.contains("- 1, x") && s.contains("- 2, y"));
    Ok(())
}

#[test]
fn strict_indent_error_alloc() {
    let s = "a:\n    b: 1\n"; // invalid +4
    let mut opts = toon::Options::default();
    opts.strict = true;
    let err = toon::de::from_str::<serde::de::IgnoredAny>(s, &opts).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("indent") || msg.contains("syntax"));
}
