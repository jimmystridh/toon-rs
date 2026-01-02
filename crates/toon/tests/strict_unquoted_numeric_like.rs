#![cfg(feature = "json")]
use toon_rs::Options;

#[test]
fn strict_unquoted_numeric_like_in_header_errors() {
    // Header token "true" must be quoted in strict mode
    let s = "rows:\n  @, true\n  - true\n";
    let mut opts = Options::default();
    opts.strict = true;
    let err = toon_rs::decode_from_str::<serde_json::Value>(s, &opts).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unquoted") || msg.contains("syntax"));
}

#[test]
fn strict_unquoted_numeric_like_cell_errors() {
    // Row cell "+1" must be quoted in strict mode
    let s = "rows:\n  @, s\n  - +1\n";
    let mut opts = Options::default();
    opts.strict = true;
    let err = toon_rs::decode_from_str::<serde_json::Value>(s, &opts).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unquoted") || msg.contains("syntax"));
}
