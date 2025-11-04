#[test]
fn options_defaults() {
    let opts = toon::Options::default();
    assert!(!opts.strict);
    assert!(matches!(opts.delimiter, toon::Delimiter::Comma));
}
