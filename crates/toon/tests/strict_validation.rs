use toon_rs::Options;

#[test]
fn strict_indent_increase_error() {
    let s = "a:\n    b: 1\n"; // 4-space indent jump instead of +2
    let mut opts = Options::default();
    opts.strict = true;
    let err = toon_rs::decode_from_str::<serde_json::Value>(s, &opts).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("indent increase") || msg.contains("syntax"));
}
