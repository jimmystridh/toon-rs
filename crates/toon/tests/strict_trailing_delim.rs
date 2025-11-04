#![cfg(feature = "json")]
use toon::Options;

#[test]
fn strict_trailing_delimiter_in_row_errors() {
    let s = "d:\n  @, a, b\n  - 1, 2,\n"; // trailing comma
    let mut opts = Options::default();
    opts.strict = true;
    let err = toon::decode_from_str::<serde_json::Value>(s, &opts).unwrap_err();
    assert!(err.to_string().contains("trailing delimiter"));
}
