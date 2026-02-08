#![cfg(all(feature = "serde", not(feature = "json")))]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Row {
    a: u32,
    b: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Wrapper {
    rows: Vec<Row>,
}

#[test]
fn typed_roundtrip_alloc() -> Result<(), toon_rs::Error> {
    let w = Wrapper {
        rows: vec![
            Row {
                a: 1,
                b: "x".into(),
            },
            Row {
                a: 2,
                b: "y".into(),
            },
        ],
    };
    let opts = toon_rs::Options::default();
    let s = toon_rs::ser::to_string_streaming(&w, &opts)?;
    let back: Wrapper = toon_rs::de::from_str(&s, &opts)?;
    assert_eq!(w, back);
    Ok(())
}

#[test]
fn tabular_emission_alloc() -> Result<(), toon_rs::Error> {
    let w = Wrapper {
        rows: vec![
            Row {
                a: 1,
                b: "x".into(),
            },
            Row {
                a: 2,
                b: "y".into(),
            },
        ],
    };
    let s = toon_rs::ser::to_string_streaming(&w, &toon_rs::Options::default())?;
    // v3.0 format: [N]{fields}: with inline rows (no list markers)
    assert!(s.contains("[2]{a,b}:"));
    assert!(s.contains("1,x") && s.contains("2,y"));
    Ok(())
}

#[test]
fn strict_indent_error_alloc() {
    let s = "a:\n   b: 1\n"; // 3-space indent is not a multiple of 2
    let opts = toon_rs::Options::default();
    let err = toon_rs::de::from_str::<serde::de::IgnoredAny>(s, &opts).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("indent") || msg.contains("multiple") || msg.contains("syntax"));
}
