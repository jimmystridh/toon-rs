#![cfg(feature = "json")]
use toon_rs::Options;

#[test]
fn strict_blank_line_in_table_error() {
    let s = "rows:\n  @, a, b\n\n  - 1, 2\n"; // blank line at table indent
    let opts = Options {
        strict: true,
        ..Options::default()
    };
    let err = toon_rs::decode_from_str::<serde_json::Value>(s, &opts).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("blank line inside table") || msg.contains("syntax"));
}
