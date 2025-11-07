use serde_json::Value;
use toon::{Delimiter, Options};
use wasm_bindgen::prelude::*;

/// Use wee_alloc as the global allocator for smaller WASM binary size
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Maximum input size in bytes (10 MB)
const MAX_INPUT_SIZE: usize = 10 * 1024 * 1024;

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
    // Validate input size
    if json_str.len() > MAX_INPUT_SIZE {
        return Err(format!(
            "Input exceeds maximum size limit of {} bytes",
            MAX_INPUT_SIZE
        ));
    }

    // Parse JSON
    let value: Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))?;

    // Configure options
    let options = Options {
        delimiter: if use_pipe_delimiter {
            Delimiter::Pipe
        } else {
            Delimiter::Comma
        },
        strict,
    };

    // Encode to TOON
    toon::encode_to_string(&value, &options).map_err(|e| format!("TOON encoding error: {}", e))
}

/// Convert TOON string to JSON format
#[wasm_bindgen]
pub fn toon_to_json(toon_str: &str, strict: bool, pretty: bool) -> Result<String, String> {
    // Validate input size
    if toon_str.len() > MAX_INPUT_SIZE {
        return Err(format!(
            "Input exceeds maximum size limit of {} bytes",
            MAX_INPUT_SIZE
        ));
    }

    // Configure options
    let options = Options {
        delimiter: Delimiter::Comma, // Delimiter is auto-detected during decode
        strict,
    };

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
