//! Path expansion for dotted keys in TOON decoding.
//!
//! When `expandPaths` is set to `safe`, dotted keys like `a.b.c` are expanded
//! into nested objects: `{ "a": { "b": { "c": ... } } }`.
//!
//! Rules:
//! - Only unquoted keys with valid identifier segments are expanded
//! - Quoted keys (e.g. "a.b") are preserved as-is
//! - Keys with non-identifier characters in segments (e.g. `full-name.x`) are preserved
//! - Deep merging is applied when multiple keys share a prefix
//! - In strict mode, conflicts (object vs primitive/array) cause an error
//! - In non-strict mode, later keys overwrite earlier ones (LWW)

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::value::Value;

/// Check if a string is a valid identifier segment for path expansion.
/// Valid identifiers contain only ASCII letters, digits, and underscores,
/// and don't start with a digit.
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let bytes = s.as_bytes();
    // First character: letter or underscore
    let first = bytes[0];
    if !first.is_ascii_alphabetic() && first != b'_' {
        return false;
    }
    // Rest: letters, digits, underscores
    bytes[1..]
        .iter()
        .all(|&b| b.is_ascii_alphanumeric() || b == b'_')
}

/// Marker character used to indicate a key came from a quoted string with dots.
/// This is a zero-width space (U+200B) which is stripped before output.
const QUOTED_DOT_MARKER: char = '\u{200B}';

/// Check if a key should be expanded (unquoted and all segments are valid identifiers).
fn should_expand_key(key: &str) -> bool {
    // Keys marked with the quoted-dot marker should not be expanded
    if key.starts_with(QUOTED_DOT_MARKER) {
        return false;
    }
    // Must contain at least one dot
    if !key.contains('.') {
        return false;
    }
    // All segments must be valid identifiers
    key.split('.').all(is_valid_identifier)
}

/// Split a key into segments for expansion.
fn split_key(key: &str) -> Vec<&str> {
    key.split('.').collect()
}

/// Deep merge `source` into `target`, applying path expansion rules.
/// Returns an error message if there's a conflict in strict mode.
#[allow(clippy::ptr_arg)]
fn deep_merge(
    target: &mut Vec<(String, Value)>,
    key: String,
    value: Value,
    strict: bool,
) -> Result<(), String> {
    // Find existing entry with the same key
    if let Some(idx) = target.iter().position(|(k, _)| k == &key) {
        let existing = &mut target[idx].1;

        // Check for conflicts
        match (&existing, &value) {
            (Value::Object(existing_obj), Value::Object(new_obj)) => {
                // Both are objects - deep merge
                let mut merged = existing_obj.clone();
                for (k, v) in new_obj.clone() {
                    deep_merge(&mut merged, k, v, strict)?;
                }
                *existing = Value::Object(merged);
            }
            _ => {
                // Conflict: different types or primitive/array
                if strict {
                    return Err(format!(
                        "path expansion conflict: key '{}' has conflicting types",
                        key
                    ));
                }
                // LWW: later value overwrites
                *existing = value;
            }
        }
    } else {
        // No existing entry - just add
        target.push((key, value));
    }
    Ok(())
}

/// Apply path expansion to a Value, returning the expanded value.
/// If `strict` is true, conflicts will cause an error.
pub fn expand_paths(value: Value, strict: bool) -> Result<Value, String> {
    match value {
        Value::Object(entries) => {
            let mut result: Vec<(String, Value)> = Vec::new();

            for (key, val) in entries {
                // Recursively expand nested values first
                let expanded_val = expand_paths(val, strict)?;

                if should_expand_key(&key) {
                    // Expand the dotted key
                    let segments = split_key(&key);
                    let nested = build_nested_from_segments(&segments, expanded_val);

                    // Merge into result
                    if let Value::Object(nested_entries) = nested {
                        for (k, v) in nested_entries {
                            deep_merge(&mut result, k, v, strict)?;
                        }
                    }
                } else {
                    // Strip the marker if present and keep the key as-is
                    let clean_key = if key.starts_with(QUOTED_DOT_MARKER) {
                        key[QUOTED_DOT_MARKER.len_utf8()..].to_string()
                    } else {
                        key
                    };
                    deep_merge(&mut result, clean_key, expanded_val, strict)?;
                }
            }

            Ok(Value::Object(result))
        }
        Value::Array(arr) => {
            let mut result = Vec::with_capacity(arr.len());
            for item in arr {
                result.push(expand_paths(item, strict)?);
            }
            Ok(Value::Array(result))
        }
        // Primitives pass through unchanged
        other => Ok(other),
    }
}

/// Build a nested object structure from key segments.
fn build_nested_from_segments(segments: &[&str], value: Value) -> Value {
    if segments.is_empty() {
        return value;
    }
    if segments.len() == 1 {
        return Value::Object(vec![(segments[0].to_string(), value)]);
    }
    // Build from the last segment to the first
    let inner = build_nested_from_segments(&segments[1..], value);
    Value::Object(vec![(segments[0].to_string(), inner)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Number;

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("a"));
        assert!(is_valid_identifier("abc"));
        assert!(is_valid_identifier("a1"));
        assert!(is_valid_identifier("_foo"));
        assert!(is_valid_identifier("foo_bar"));

        assert!(!is_valid_identifier("")); // empty
        assert!(!is_valid_identifier("1a")); // starts with digit
        assert!(!is_valid_identifier("full-name")); // contains hyphen
        assert!(!is_valid_identifier("a.b")); // contains dot (should be split first)
    }

    #[test]
    fn test_should_expand_key() {
        assert!(should_expand_key("a.b"));
        assert!(should_expand_key("a.b.c"));
        assert!(should_expand_key("user.name"));

        assert!(!should_expand_key("a")); // no dot
        assert!(!should_expand_key("\u{200B}a.b")); // marked as quoted
        assert!(!should_expand_key("full-name.x")); // hyphen in segment
    }

    #[test]
    fn test_expand_simple() {
        let input = Value::Object(vec![("a.b.c".to_string(), Value::Number(Number::I64(1)))]);

        let result = expand_paths(input, false).unwrap();

        let expected = Value::Object(vec![(
            "a".to_string(),
            Value::Object(vec![(
                "b".to_string(),
                Value::Object(vec![("c".to_string(), Value::Number(Number::I64(1)))]),
            )]),
        )]);

        assert_eq!(result, expected);
    }
}
