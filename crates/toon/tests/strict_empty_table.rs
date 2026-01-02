#![cfg(feature = "json")]
use toon_rs::Options;

#[test]
fn strict_empty_table_error() {
    let s = "rows:\n  @, a, b\n"; // no rows
    let mut opts = Options::default();
    opts.strict = true;
    let err = toon_rs::decode_from_str::<serde_json::Value>(s, &opts).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("empty table"));
}
