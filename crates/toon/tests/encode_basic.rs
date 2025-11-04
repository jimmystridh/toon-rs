use serde_json::json;

#[test]
fn encode_simple_object() -> Result<(), Box<dyn std::error::Error>> {
    let v = json!({"a": 1, "s": "hi"});
    let out = toon::encode_to_string(&v, &toon::Options::default())?;
    assert!(out.contains("a: 1"));
    assert!(out.contains("s: hi"));
    Ok(())
}

#[test]
fn encode_nested_array() -> Result<(), Box<dyn std::error::Error>> {
    let v = json!({"list": [1, 2, 3]});
    let out = toon::encode_to_string(&v, &toon::Options::default())?;
    assert!(out.contains("list:"));
    assert!(out.contains("- 1"));
    assert!(out.contains("- 2"));
    assert!(out.contains("- 3"));
    Ok(())
}

#[test]
fn quoting_rules_examples() -> Result<(), Box<dyn std::error::Error>> {
    let v = json!({
        "empty": "",
        "looks_bool": "true",
        "starts_dash": "- x",
        "has_colon": "a:b",
        "with_comma": ",x"
    });
    let out = toon::encode_to_string(&v, &toon::Options::default())?;
    assert!(out.contains("empty: \"\""));
    assert!(out.contains("looks_bool: \"true\""));
    assert!(out.contains("starts_dash: \"- x\""));
    assert!(out.contains("has_colon: \"a:b\""));
    assert!(out.contains("with_comma: \"\\,x\"") || out.contains("with_comma: \",x\""));
    Ok(())
}
