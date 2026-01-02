#[test]
fn options_defaults() {
    let opts = toon_rs::Options::default();
    assert!(!opts.strict);
    assert!(matches!(opts.delimiter, toon_rs::Delimiter::Comma));
}
