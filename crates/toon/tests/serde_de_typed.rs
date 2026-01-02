#![cfg(feature = "json")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct User {
    id: u32,
    name: String,
    flags: Vec<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Wrapper {
    user: User,
}

#[test]
fn de_typed_struct_via_deserializer() -> Result<(), Box<dyn std::error::Error>> {
    let s = "user:\n  id: 1\n  name: \"Ada\"\n  flags:\n    - true\n    - false\n";
    let opts = toon_rs::Options::default();
    let val: serde_json::Value = toon_rs::decode_from_str(s, &opts)?; // JSON Value path
    assert_eq!(val["user"]["id"], 1);

    // Typed path via our Deserializer
    let u: Wrapper = toon_rs::de::from_str(s, &opts)?;
    assert_eq!(
        u,
        Wrapper {
            user: User {
                id: 1,
                name: "Ada".into(),
                flags: vec![true, false]
            }
        }
    );
    Ok(())
}
