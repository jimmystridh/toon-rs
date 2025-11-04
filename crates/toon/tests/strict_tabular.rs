use toon::Options;

#[test]
fn strict_tabular_row_length_mismatch() {
    let s = "rows:\n  @, a, b\n  - 1\n"; // only 1 cell, header expects 2
    let mut opts = Options::default();
    opts.strict = true;
    let err = toon::decode_from_str::<serde_json::Value>(s, &opts).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("row cell count") || msg.contains("syntax"));
}

#[test]
fn strict_tabular_invalid_delimiter() {
    let s = "rows:\n  @; a; b\n  - 1; 2\n";
    let mut opts = Options::default();
    opts.strict = true;
    let err = toon::decode_from_str::<serde_json::Value>(s, &opts).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("invalid header delimiter") || msg.contains("syntax"));
}

#[test]
fn strict_tabular_duplicate_keys() {
    let s = "rows:\n  @, a, a\n  - 1, 2\n";
    let mut opts = Options::default();
    opts.strict = true;
    let err = toon::decode_from_str::<serde_json::Value>(s, &opts).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("duplicate header key") || msg.contains("syntax"));
}
