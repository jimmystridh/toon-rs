#![cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use toon::Options;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Row {
    a: u32,
    b: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Rows {
    rows: Vec<Row>,
}

#[test]
fn ser_typed_emits_tabular() -> Result<(), Box<dyn std::error::Error>> {
    let value = Rows {
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
    let out = toon::ser::to_string(&value, &Options::default())?;
    // Spec v3.0: tabular arrays use rows[N]{fields}: format
    assert!(out.contains("rows[2]{a,b}:"), "Output: {}", out);
    // Rows are at indent+2, values comma-separated
    assert!(
        out.contains("1,x") || out.contains("1, x"),
        "Output: {}",
        out
    );
    Ok(())
}
