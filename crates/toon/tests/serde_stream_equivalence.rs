#![cfg(feature = "json")]
use serde_json::json;

#[test]
fn streaming_and_value_paths_match() -> Result<(), Box<dyn std::error::Error>> {
    let v = json!({
        "a": 1,
        "b": [true, "x", {"c": 3}],
        "rows": [{"a":1, "b":"u"}, {"a":2, "b":"v"}]
    });
    let opts = toon_rs::Options::default();
    let s1 = toon_rs::ser::to_string(&v, &opts)?;
    let s2 = toon_rs::ser::to_string_streaming(&v, &opts)?;
    assert_eq!(s1, s2);
    Ok(())
}
