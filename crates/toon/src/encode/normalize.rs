use serde_json::Value;

// Normalization for non-streaming encode path.
// - serde_json::Value cannot represent NaN/±Infinity as numbers; callers should map them to strings.
// - Dates (chrono) are serialized by serde as strings; we leave them intact here.
// This pass currently returns the input, but kept for future policy hooks.
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
