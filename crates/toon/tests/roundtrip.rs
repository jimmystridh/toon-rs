use serde_json::json;

#[test]
fn encode_outputs_lines() -> Result<(), Box<dyn std::error::Error>> {
    let value = json!({"a": 1, "b": [true, "x"]});
    let options = toon::Options::default();

    let s = toon::encode_to_string(&value, &options)?;
    assert!(s.contains("a: 1"));
    assert!(s.contains("b:"));
    assert!(s.contains("- true"));
    Ok(())
}

#[test]
fn decode_basic_toon() -> Result<(), Box<dyn std::error::Error>> {
    let options = toon::Options::default();
    let s = "a: 1\nb:\n  - true\n  - \"x\"\n";
    let v: serde_json::Value = toon::decode_from_str(s, &options)?;
    assert_eq!(v, json!({"a":1, "b": [true, "x"]}));
    Ok(())
}
