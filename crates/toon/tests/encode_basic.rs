#![cfg(feature = "json")]
use serde_json::json;

#[test]
fn encode_simple_object() -> Result<(), Box<dyn std::error::Error>> {
    let v = json!({"a": 1, "s": "hi"});
    let out = toon_rs::encode_to_string(&v, &toon_rs::Options::default())?;
    assert!(out.contains("a: 1"));
    assert!(out.contains("s: hi"));
    Ok(())
}

#[test]
fn encode_nested_array() -> Result<(), Box<dyn std::error::Error>> {
    let v = json!({"list": [1, 2, 3]});
    let out = toon_rs::encode_to_string(&v, &toon_rs::Options::default())?;
    // Spec v3.0: inline primitive arrays use key[N]: v1,v2,v3 format
    assert!(out.contains("list[3]"), "Output: {}", out);
    assert!(out.contains("1,2,3"), "Output: {}", out);
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
    let out = toon_rs::encode_to_string(&v, &toon_rs::Options::default())?;
    assert!(out.contains("empty: \"\""));
    assert!(out.contains("looks_bool: \"true\""));
    assert!(out.contains("starts_dash: \"- x\""));
    assert!(out.contains("has_colon: \"a:b\""));
    assert!(out.contains("with_comma: \"\\,x\"") || out.contains("with_comma: \",x\""));
    Ok(())
}
