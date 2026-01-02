#![cfg(feature = "json")]
use serde_json::json;

#[test]
fn encode_tabular_emits_header_and_rows() -> Result<(), Box<dyn std::error::Error>> {
    let v = json!({
        "rows": [
            {"a": 1, "b": "x"},
            {"a": 2, "b": "y"}
        ]
    });
    let out = toon_rs::encode_to_string(&v, &toon_rs::Options::default())?;
    // Header (spec v3): rows[2]{a,b}:
    assert!(out.contains("rows[2]{a,b}:"));
    // Rows (no hyphens; rows follow header at indent+2)
    assert!(out.contains("\n  1,x\n"));
    // Last row has no trailing newline (per spec §12)
    assert!(out.ends_with("\n  2,y"));
    Ok(())
}

#[test]
fn decode_tabular_to_objects() -> Result<(), Box<dyn std::error::Error>> {
    // Spec v3: keyed tabular array
    let s = "rows[2]{a,b}:\n  1,x\n  2,y\n";
    let val: serde_json::Value = toon_rs::decode_from_str(s, &toon_rs::Options::default())?;
    assert_eq!(val, json!({"rows": [{"a":1, "b":"x"}, {"a":2, "b":"y"}]}));
    Ok(())
}
