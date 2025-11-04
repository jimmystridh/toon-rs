use serde::{Deserialize, Serialize};
use toon::Options;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Data {
    s: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Wrapper { d: Vec<Data> }

#[test]
fn unquoting_preserves_strings_with_delimiters_and_colons() {
    let s = "d:\n  @, s\n  - \"a,b:c\"\n";
    let out: Wrapper = toon::de::from_str(s, &Options::default()).unwrap();
    assert_eq!(out, Wrapper { d: vec![Data { s: "a,b:c".into() }] });
}
