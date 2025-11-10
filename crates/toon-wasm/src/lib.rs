use serde_json::Value;
use serde_wasm_bindgen::{from_value as from_js_value, to_value as to_js_value};
use toon::{Delimiter, Options};
use wasm_bindgen::prelude::*;

/// Use wee_alloc as the global allocator only when the optional feature is
/// enabled. The default build now favors runtime performance.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Maximum input size in bytes (10 MB)
const MAX_INPUT_SIZE: usize = 10 * 1024 * 1024;

fn limit_error_message() -> String {
    format!(
        "Input exceeds maximum size limit of {} bytes",
        MAX_INPUT_SIZE
    )
}

fn limit_error_js() -> JsValue {
    JsValue::from_str(&limit_error_message())
}

fn options_for_encode(use_pipe_delimiter: bool, strict: bool) -> Options {
    Options {
        delimiter: if use_pipe_delimiter {
            Delimiter::Pipe
        } else {
            Delimiter::Comma
        },
        strict,
    }
}

fn options_for_decode(strict: bool) -> Options {
    Options {
        delimiter: Delimiter::Comma,
        strict,
    }
}

fn js_error(err: impl core::fmt::Display) -> JsValue {
    JsValue::from_str(&err.to_string())
}

fn estimated_value_size(value: &Value) -> usize {
    match value {
        Value::Null => 4,
        Value::Bool(_) => 5,
        Value::Number(n) => estimate_number_len(n),
        Value::String(s) => s.len(),
        Value::Array(items) => {
            // Count brackets plus elements
            2 + items.iter().map(estimated_value_size).sum::<usize>()
        }
        Value::Object(obj) => {
            // Account for braces plus keys and values
            2 + obj
                .iter()
                .map(|(k, v)| k.len() + estimated_value_size(v))
                .sum::<usize>()
        }
    }
}

fn estimate_number_len(num: &serde_json::Number) -> usize {
    if let Some(i) = num.as_i64() {
        digits_i64(i)
    } else if let Some(u) = num.as_u64() {
        digits_u64(u)
    } else if let Some(f) = num.as_f64() {
        // This only happens for non-integer numbers; we fall back to a string
        // allocation, but the size check still avoids creating a large JSON
        // payload eagerly.
        f.to_string().len()
    } else {
        0
    }
}

fn digits_i64(value: i64) -> usize {
    if value == 0 {
        return 1;
    }
    let mut len = 0;
    let mut v = value as i128;
    if v < 0 {
        len += 1; // minus sign
        v = -v;
    }
    while v > 0 {
        len += 1;
        v /= 10;
    }
    len
}

fn digits_u64(mut value: u64) -> usize {
    if value == 0 {
        return 1;
    }
    let mut len = 0;
    while value > 0 {
        len += 1;
        value /= 10;
    }
    len
}

/// Initialize panic hook for better error messages in browser console.
/// Call this once when the module is loaded for improved debugging.
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Convert JSON string to TOON format
#[wasm_bindgen]
pub fn json_to_toon(
    json_str: &str,
    use_pipe_delimiter: bool,
    strict: bool,
) -> Result<String, String> {
    if json_str.len() > MAX_INPUT_SIZE {
        return Err(limit_error_message());
    }

    // Parse JSON
    let value: Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;

    // Configure options
    let options = options_for_encode(use_pipe_delimiter, strict);

    // Encode to TOON
    toon::encode_to_string(&value, &options).map_err(|e| format!("TOON encoding error: {}", e))
}

/// Convert TOON string to JSON format
#[wasm_bindgen]
pub fn toon_to_json(toon_str: &str, strict: bool, pretty: bool) -> Result<String, String> {
    if toon_str.len() > MAX_INPUT_SIZE {
        return Err(limit_error_message());
    }

    let options = options_for_decode(strict);

    // Decode from TOON
    let value: Value = toon::decode_from_str(toon_str, &options)
        .map_err(|e| format!("TOON decoding error: {}", e))?;

    // Convert to JSON string
    if pretty {
        serde_json::to_string_pretty(&value).map_err(|e| format!("JSON encoding error: {}", e))
    } else {
        serde_json::to_string(&value).map_err(|e| format!("JSON encoding error: {}", e))
    }
}

/// Convert an in-memory JavaScript value to TOON without going through
/// `JSON.stringify` first.
#[wasm_bindgen]
pub fn value_to_toon(
    value: JsValue,
    use_pipe_delimiter: bool,
    strict: bool,
) -> Result<String, JsValue> {
    let value: Value = from_js_value(value).map_err(js_error)?;
    if estimated_value_size(&value) > MAX_INPUT_SIZE {
        return Err(limit_error_js());
    }
    let options = options_for_encode(use_pipe_delimiter, strict);
    toon::encode_to_string(&value, &options).map_err(js_error)
}

/// Decode TOON into a JavaScript value so callers can defer stringifying to the
/// host runtime.
#[wasm_bindgen]
pub fn toon_to_value(toon_str: &str, strict: bool) -> Result<JsValue, JsValue> {
    if toon_str.len() > MAX_INPUT_SIZE {
        return Err(limit_error_js());
    }
    let options = options_for_decode(strict);
    let value: Value = toon::decode_from_str(toon_str, &options).map_err(js_error)?;
    to_js_value(&value).map_err(js_error)
}

/// Get the version of the TOON library
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_toon_simple() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let result = json_to_toon(json, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_toon_to_json_simple() {
        let toon = "name: Alice\nage: 30";
        let result = toon_to_json(toon, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_roundtrip() {
        let json = r#"{"name": "Bob", "values": [1, 2, 3]}"#;
        let toon = json_to_toon(json, false, false).unwrap();
        let json_back = toon_to_json(&toon, false, true).unwrap();

        // Parse both to compare values
        let original: serde_json::Value = serde_json::from_str(json).unwrap();
        let roundtrip: serde_json::Value = serde_json::from_str(&json_back).unwrap();
        assert_eq!(original, roundtrip);
    }

    #[test]
    fn test_json_to_toon_size_limit() {
        // Create a JSON string that exceeds MAX_INPUT_SIZE
        let large_json = "x".repeat(MAX_INPUT_SIZE + 1);
        let result = json_to_toon(&large_json, false, false);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Input exceeds maximum size limit")
        );
    }

    #[test]
    fn test_toon_to_json_size_limit() {
        // Create a TOON string that exceeds MAX_INPUT_SIZE
        let large_toon = "x".repeat(MAX_INPUT_SIZE + 1);
        let result = toon_to_json(&large_toon, false, false);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Input exceeds maximum size limit")
        );
    }

    // Edge case tests

    #[test]
    fn test_empty_string_json_to_toon() {
        let result = json_to_toon("", false, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    #[test]
    fn test_empty_string_toon_to_json() {
        let result = toon_to_json("", false, false);
        // Empty string may be valid TOON (empty document)
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_empty_object() {
        let json = "{}";
        let result = json_to_toon(json, false, false);
        assert!(result.is_ok());
        let toon = result.unwrap();
        let json_back = toon_to_json(&toon, false, false).unwrap();
        assert_eq!(json_back, "{}");
    }

    #[test]
    fn test_empty_array() {
        let json = "[]";
        let result = json_to_toon(json, false, false);
        assert!(result.is_ok());
        let toon = result.unwrap();
        let json_back = toon_to_json(&toon, false, false).unwrap();
        assert_eq!(json_back, "[]");
    }

    #[test]
    fn test_null_value() {
        let json = r#"{"value": null}"#;
        let result = json_to_toon(json, false, false);
        assert!(result.is_ok());
        let toon = result.unwrap();
        let json_back = toon_to_json(&toon, false, true).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_back).unwrap();
        assert!(parsed["value"].is_null());
    }

    #[test]
    fn test_nested_null_values() {
        let json = r#"{"outer": {"inner": null, "value": 42}}"#;
        let result = json_to_toon(json, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let invalid = "{not valid json}";
        let result = json_to_toon(invalid, false, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    #[test]
    fn test_invalid_toon() {
        // This may actually be valid TOON (multiple key-value pairs)
        // Let's test with something truly invalid
        let invalid = ":::invalid:::";
        let result = toon_to_json(invalid, false, false);
        // Just verify it handles it without panicking
        let _ = result;
    }

    #[test]
    fn test_unicode_handling() {
        let json = r#"{"emoji": "🎨", "chinese": "你好", "arabic": "مرحبا"}"#;
        let toon = json_to_toon(json, false, false).unwrap();
        let json_back = toon_to_json(&toon, false, false).unwrap();
        let original: serde_json::Value = serde_json::from_str(json).unwrap();
        let roundtrip: serde_json::Value = serde_json::from_str(&json_back).unwrap();
        assert_eq!(original, roundtrip);
    }

    #[test]
    fn test_special_characters() {
        let json = r#"{"quote": "\"", "backslash": "\\", "newline": "\n", "tab": "\t"}"#;
        let toon = json_to_toon(json, false, false).unwrap();
        let json_back = toon_to_json(&toon, false, false).unwrap();
        let original: serde_json::Value = serde_json::from_str(json).unwrap();
        let roundtrip: serde_json::Value = serde_json::from_str(&json_back).unwrap();
        assert_eq!(original, roundtrip);
    }

    #[test]
    fn test_numbers_edge_cases() {
        let json = r#"{"zero": 0, "negative": -42, "float": 3.14159, "large": 9007199254740991}"#;
        let toon = json_to_toon(json, false, false).unwrap();
        let json_back = toon_to_json(&toon, false, false).unwrap();
        let original: serde_json::Value = serde_json::from_str(json).unwrap();
        let roundtrip: serde_json::Value = serde_json::from_str(&json_back).unwrap();
        assert_eq!(original, roundtrip);
    }

    #[test]
    fn test_boolean_values() {
        let json = r#"{"true_val": true, "false_val": false}"#;
        let toon = json_to_toon(json, false, false).unwrap();
        let json_back = toon_to_json(&toon, false, false).unwrap();
        let original: serde_json::Value = serde_json::from_str(json).unwrap();
        let roundtrip: serde_json::Value = serde_json::from_str(&json_back).unwrap();
        assert_eq!(original, roundtrip);
    }

    #[test]
    fn test_deeply_nested_structure() {
        let json = r#"{"a": {"b": {"c": {"d": {"e": "deep"}}}}}"#;
        let toon = json_to_toon(json, false, false).unwrap();
        let json_back = toon_to_json(&toon, false, false).unwrap();
        let original: serde_json::Value = serde_json::from_str(json).unwrap();
        let roundtrip: serde_json::Value = serde_json::from_str(&json_back).unwrap();
        assert_eq!(original, roundtrip);
    }

    #[test]
    fn test_mixed_array() {
        let json = r#"{"mixed": [1, "two", true, null, {"nested": "object"}]}"#;
        let toon = json_to_toon(json, false, false).unwrap();
        let json_back = toon_to_json(&toon, false, false).unwrap();
        let original: serde_json::Value = serde_json::from_str(json).unwrap();
        let roundtrip: serde_json::Value = serde_json::from_str(&json_back).unwrap();
        assert_eq!(original, roundtrip);
    }

    #[test]
    fn test_pipe_vs_comma_delimiter() {
        let json = r#"{"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}"#;
        let toon_pipe = json_to_toon(json, true, false).unwrap();
        let toon_comma = json_to_toon(json, false, false).unwrap();
        // Both should decode to the same JSON
        let json_from_pipe = toon_to_json(&toon_pipe, false, false).unwrap();
        let json_from_comma = toon_to_json(&toon_comma, false, false).unwrap();
        let from_pipe: serde_json::Value = serde_json::from_str(&json_from_pipe).unwrap();
        let from_comma: serde_json::Value = serde_json::from_str(&json_from_comma).unwrap();
        assert_eq!(from_pipe, from_comma);
    }

    #[test]
    fn test_strict_mode_roundtrip() {
        let json = r#"{"name": "test", "value": 123}"#;
        let toon = json_to_toon(json, false, true).unwrap();
        let result = toon_to_json(&toon, true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pretty_json_formatting() {
        let json = r#"{"a":1,"b":2}"#;
        let toon = json_to_toon(json, false, false).unwrap();
        let pretty = toon_to_json(&toon, false, true).unwrap();
        let compact = toon_to_json(&toon, false, false).unwrap();
        // Pretty should have newlines and indentation
        assert!(pretty.contains('\n'));
        assert!(!compact.contains('\n'));
    }

    // Large input tests

    #[test]
    fn test_large_but_valid_array() {
        // Create a large array that's under the size limit
        let items: Vec<i32> = (0..1000).collect();
        let json = serde_json::to_string(&items).unwrap();
        assert!(json.len() < MAX_INPUT_SIZE);
        let result = json_to_toon(&json, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_large_but_valid_object() {
        // Create a large object that's under the size limit
        let mut obj = serde_json::Map::new();
        for i in 0..100 {
            obj.insert(
                format!("key_{}", i),
                serde_json::json!(format!("value_{}", i)),
            );
        }
        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.len() < MAX_INPUT_SIZE);
        let result = json_to_toon(&json, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_whitespace_handling() {
        let json_compact = r#"{"a":1,"b":2}"#;
        let json_spaced = r#"{ "a" : 1 , "b" : 2 }"#;
        let toon1 = json_to_toon(json_compact, false, false).unwrap();
        let toon2 = json_to_toon(json_spaced, false, false).unwrap();
        // Both should produce equivalent TOON output
        let back1: serde_json::Value =
            serde_json::from_str(&toon_to_json(&toon1, false, false).unwrap()).unwrap();
        let back2: serde_json::Value =
            serde_json::from_str(&toon_to_json(&toon2, false, false).unwrap()).unwrap();
        assert_eq!(back1, back2);
    }
}
