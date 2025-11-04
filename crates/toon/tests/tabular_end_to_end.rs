use serde_json::json;

#[test]
fn encode_tabular_emits_header_and_rows() -> Result<(), Box<dyn std::error::Error>> {
    let v = json!({
        "rows": [
            {"a": 1, "b": "x"},
            {"a": 2, "b": "y"}
        ]
    });
    let out = toon::encode_to_string(&v, &toon::Options::default())?;
    // Header
    assert!(out.contains("@, a, b") || out.contains("@, \"a\", \"b\""));
    // Rows
    assert!(out.contains("- 1, \"x\"") || out.contains("- 1, x"));
    assert!(out.contains("- 2, \"y\"") || out.contains("- 2, y"));
    Ok(())
}

#[test]
fn decode_tabular_to_objects() -> Result<(), Box<dyn std::error::Error>> {
    let s = "rows:\n  @, a, b\n  - 1, \"x\"\n  - 2, \"y\"\n";
    let val: serde_json::Value = toon::decode_from_str(s, &toon::Options::default())?;
    assert_eq!(val, json!({"rows": [{"a":1, "b":"x"}, {"a":2, "b":"y"}]}));
    Ok(())
}
