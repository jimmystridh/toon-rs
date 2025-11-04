#![cfg(feature = "json")]
use serde_json::json;
use toon::encode::encoders::is_tabular_array;

#[test]
fn tabular_detection_positive() {
    let arr = vec![
        json!({"a": 1, "b": "x"}),
        json!({"b": "y", "a": 2}),
    ];
    let keys = is_tabular_array(&arr).expect("should be tabular");
    assert_eq!(keys, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn tabular_detection_negative_mixed_keys() {
    let arr = vec![
        json!({"a": 1, "b": "x"}),
        json!({"a": 2, "c": 3}),
    ];
    assert!(is_tabular_array(&arr).is_none());
}

#[test]
fn tabular_detection_negative_nested_values() {
    let arr = vec![
        json!({"a": 1, "b": {"x": 1}}),
        json!({"a": 2, "b": {"x": 2}}),
    ];
    assert!(is_tabular_array(&arr).is_none());
}
