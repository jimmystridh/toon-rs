#![cfg(feature = "json")]
use serde_json::json;
use toon::decode::parser::parse_to_value;

#[test]
fn parse_object_and_list() {
    let input = "a: 1\nb:\n  - true\n  - \"x\"\n";
    let v = parse_to_value(input);
    assert_eq!(v, json!({"a":1,"b":[true,"x"]}));
}

#[test]
fn parse_nested_in_list_item() {
    let input = "list:\n  -\n    a: 1\n    b: 2\n  - 3\n";
    let v = parse_to_value(input);
    assert_eq!(v, json!({"list": [{"a":1,"b":2}, 3]}));
}
