#![cfg(feature = "json")]
use toon_rs::Options;

#[test]
fn quote_numeric_like_strings() {
    let v = serde_json::json!({
        "n1": "01",
        "n2": "+1.2",
        "n3": "-3.4e5",
        "b": "true",
        "nulls": "null",
    });
    let out = toon_rs::encode_to_string(&v, &Options::default()).unwrap();
    assert!(out.contains("n1: \"01\""));
    assert!(out.contains("n2: \"+1.2\""));
    assert!(out.contains("n3: \"-3.4e5\""));
    assert!(out.contains("b: \"true\""));
    assert!(out.contains("nulls: \"null\""));
}
