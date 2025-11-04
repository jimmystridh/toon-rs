use serde_json::Value;

// Placeholder normalization: recursively clones values.
// Future: map NaN/Infinity to strings, handle date/time formatting, etc.
pub fn normalize_value(v: &Value) -> Value {
    match v {
        Value::Null => Value::Null,
        Value::Bool(b) => Value::Bool(*b),
        Value::Number(n) => Value::Number(n.clone()),
        Value::String(s) => Value::String(s.clone()),
        Value::Array(a) => Value::Array(a.iter().map(normalize_value).collect()),
        Value::Object(m) => {
            let mut out = serde_json::Map::new();
            for (k, vv) in m.iter() {
                out.insert(k.clone(), normalize_value(vv));
            }
            Value::Object(out)
        }
    }
}
