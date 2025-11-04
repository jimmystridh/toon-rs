#![cfg(feature = "json")]
use serde::{Serialize, Deserialize};
use toon::Options;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Row { a: u32, b: String }

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Rows { rows: Vec<Row> }

#[test]
fn ser_typed_emits_tabular() -> Result<(), Box<dyn std::error::Error>> {
    let value = Rows { rows: vec![ Row { a: 1, b: "x".into() }, Row { a: 2, b: "y".into() } ] };
    let out = toon::ser::to_string(&value, &Options::default())?;
    assert!(out.contains("@, a, b") || out.contains("@, \"a\", \"b\""));
    assert!(out.contains("- 1, x") || out.contains("- 1, \"x\""));
    Ok(())
}
