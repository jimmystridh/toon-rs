#![cfg(feature = "json")]
use toon_rs::Options;

#[test]
fn encoder_quotes_numeric_bool_null_lookalikes() {
    let v = serde_json::json!({
        "t": "true",
        "f": "false",
        "n": "null",
        "i": "+1",
        "f2": "-3.14",
        "z": "01"
    });
    let out = toon_rs::encode_to_string(&v, &Options::default()).unwrap();
    assert!(out.contains("t: \"true\""));
    assert!(out.contains("f: \"false\""));
    assert!(out.contains("n: \"null\""));
    assert!(out.contains("i: \"+1\""));
    assert!(out.contains("f2: \"-3.14\""));
    assert!(out.contains("z: \"01\""));
}
