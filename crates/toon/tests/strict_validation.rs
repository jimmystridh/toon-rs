use toon_rs::Options;

#[test]
fn strict_non_multiple_indent_error() {
    let s = "a:\n   b: 1\n"; // 3-space indent is not a multiple of 2
    let opts = Options::default();
    let err = toon_rs::decode_from_str::<serde_json::Value>(s, &opts).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("multiple") || msg.contains("indent") || msg.contains("syntax"));
}
