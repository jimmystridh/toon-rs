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
}
