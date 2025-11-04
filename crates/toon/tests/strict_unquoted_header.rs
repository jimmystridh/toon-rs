use toon::Options;

#[test]
fn strict_unquoted_header_token_errors() {
    let s = "rows:\n  @, a:b, c\n  - 1, 2\n"; // header token contains colon but is unquoted
    let mut opts = Options::default();
    opts.strict = true;
    let err = toon::decode_from_str::<serde_json::Value>(s, &opts).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unquoted header token") || msg.contains("syntax"));
}
