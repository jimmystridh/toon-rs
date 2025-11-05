#![cfg(feature = "json")]
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

#[test]
fn encode_empty_array() -> Result<(), Box<dyn std::error::Error>> {
    let value = json!([]);
    let options = toon::Options::default();
    let s = toon::encode_to_string(&value, &options)?;
    assert_eq!(s.trim(), "[0]:");
    Ok(())
}

#[test]
fn decode_empty_array() -> Result<(), Box<dyn std::error::Error>> {
    let options = toon::Options::default();
    let s = "[0]:";
    let v: serde_json::Value = toon::decode_from_str(s, &options)?;
    assert_eq!(v, json!([]));
    Ok(())
}

#[test]
fn roundtrip_empty_array() -> Result<(), Box<dyn std::error::Error>> {
    let original = json!([]);
    let options = toon::Options::default();

    let encoded = toon::encode_to_string(&original, &options)?;
    let decoded: serde_json::Value = toon::decode_from_str(&encoded, &options)?;

    assert_eq!(original, decoded);
    Ok(())
}

#[test]
fn canonical_number_formatting() -> Result<(), Box<dyn std::error::Error>> {
    let value = json!({"x": 0.0, "y": 1.0, "z": 1.5, "neg": -0.5});
    let options = toon::Options::default();
    let s = toon::encode_to_string(&value, &options)?;
    assert!(s.contains("x: 0"));
    assert!(s.contains("y: 1"));
    assert!(s.contains("z: 1.5"));
    assert!(s.contains("neg: -0.5"));
    Ok(())
}

#[test]
fn roundtrip_floats_preserves_decimals() -> Result<(), Box<dyn std::error::Error>> {
    let original = json!({"a": 0.0, "b": 1.0, "c": 1.5});
    let options = toon::Options::default();

    let encoded = toon::encode_to_string(&original, &options)?;
    let decoded: serde_json::Value = toon::decode_from_str(&encoded, &options)?;

    assert_eq!(decoded, json!({"a": 0, "b": 1, "c": 1.5}));
    Ok(())
}

#[test]
fn roundtrip_root_float() -> Result<(), Box<dyn std::error::Error>> {
    let original = json!(0.0);
    let options = toon::Options::default();

    let encoded = toon::encode_to_string(&original, &options)?;
    assert_eq!(encoded.trim(), "0");

    let decoded: serde_json::Value = toon::decode_from_str(&encoded, &options)?;
    assert_eq!(decoded, json!(0));
    Ok(())
}

#[test]
fn decode_empty_string_as_empty_object() -> Result<(), Box<dyn std::error::Error>> {
    let options = toon::Options::default();
    let s = "";
    let v: serde_json::Value = toon::decode_from_str(s, &options)?;
    assert_eq!(v, json!({}));
    Ok(())
}

#[test]
fn roundtrip_empty_object() -> Result<(), Box<dyn std::error::Error>> {
    let original = json!({});
    let options = toon::Options::default();

    let encoded = toon::encode_to_string(&original, &options)?;
    let decoded: serde_json::Value = toon::decode_from_str(&encoded, &options)?;

    assert_eq!(original, decoded);
    Ok(())
}

#[test]
fn roundtrip_null_preserves_null() -> Result<(), Box<dyn std::error::Error>> {
    let original = json!(null);
    let options = toon::Options::default();

    let encoded = toon::encode_to_string(&original, &options)?;
    assert_eq!(encoded.trim(), "null");

    let decoded: serde_json::Value = toon::decode_from_str(&encoded, &options)?;
    assert_eq!(original, decoded);
    Ok(())
}

// Bug #8: Empty string keys in tabular arrays
#[test]
fn roundtrip_tabular_array_with_empty_string_key() -> Result<(), Box<dyn std::error::Error>> {
    let original = json!([{"": null}]);
    let options = toon::Options::default();

    let encoded = toon::encode_to_string(&original, &options)?;
    println!("Encoded TOON:\n{}", encoded);

    let decoded: serde_json::Value = toon::decode_from_str(&encoded, &options)?;
    println!("Decoded JSON: {}", decoded);

    // This should roundtrip correctly
    assert_eq!(
        original, decoded,
        "Tabular array with empty string key should roundtrip correctly"
    );
    Ok(())
}

// Bug #8: Multiple rows with empty string key
#[test]
fn roundtrip_tabular_array_with_empty_string_key_multiple_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let original = json!([{"": "a"}, {"": "b"}]);
    let options = toon::Options::default();

    let encoded = toon::encode_to_string(&original, &options)?;
    let decoded: serde_json::Value = toon::decode_from_str(&encoded, &options)?;

    assert_eq!(original, decoded);
    Ok(())
}

// Bug #8: Mixed keys including empty string
#[test]
fn roundtrip_tabular_array_with_mixed_keys_including_empty()
-> Result<(), Box<dyn std::error::Error>> {
    let original = json!([{"": "x", "name": "test"}]);
    let options = toon::Options::default();

    let encoded = toon::encode_to_string(&original, &options)?;
    let decoded: serde_json::Value = toon::decode_from_str(&encoded, &options)?;

    assert_eq!(original, decoded);
    Ok(())
}

#[test]
fn roundtrip_object_value_with_unicode_whitespace() -> Result<(), Box<dyn std::error::Error>> {
    let original = json!([{"": null, "1": "\u{0085}"}]);
    let options = toon::Options::default();

    let encoded = toon::encode_to_string(&original, &options)?;
    let decoded: serde_json::Value = toon::decode_from_str(&encoded, &options)?;

    assert_eq!(original, decoded);
    Ok(())
}

#[test]
fn roundtrip_tabular_header_with_unicode_whitespace() -> Result<(), Box<dyn std::error::Error>> {
    let original = json!([[{"\u{2001}": null}]]);
    let options = toon::Options::default();

    let encoded = toon::encode_to_string(&original, &options)?;
    let decoded: serde_json::Value = toon::decode_from_str(&encoded, &options)?;

    assert_eq!(original, decoded);
    Ok(())
}
