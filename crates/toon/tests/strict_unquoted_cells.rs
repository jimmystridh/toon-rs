#![cfg(feature = "json")]
use toon::Options;

#[test]
fn strict_unquoted_cell_with_delimiter_errors() {
    let s = "d:\n  @, s\n  - a,b\n"; // cell with a comma must be quoted
    let mut opts = Options::default();
    opts.strict = true;
    let err = toon::decode_from_str::<serde_json::Value>(s, &opts).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unquoted cell") || msg.contains("syntax"));
}
