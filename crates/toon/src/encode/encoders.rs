use crate::{
    Result,
    encode::{primitives, writer::LineWriter},
    options::Options,
};

#[cfg(feature = "serde")]
use serde_json::Value;

pub fn encode_value(
    value: &Value,
    w: &mut LineWriter,
    opts: &Options,
    indent: usize,
) -> Result<()> {
    match value {
        Value::Null => w.line(indent, primitives::format_null()),
        Value::Bool(b) => w.line(indent, primitives::format_bool(*b)),
        Value::Number(n) => w.line(indent, &n.to_string()),
        Value::String(s) => {
            let fs = primitives::format_string(s, opts.delimiter);
            w.line(indent, &fs)
        }
        Value::Array(items) => {
            if let Some(keys) = is_tabular_array(items) {
                // Emit header and rows using active delimiter
                let dch = primitives::delimiter_char(opts.delimiter);
                let key_cells: Vec<String> = keys
                    .iter()
                    .map(|k| primitives::format_string(k, opts.delimiter))
                    .collect();
                let header = join_with_delim(&key_cells, dch);
                w.line(indent, &format!("@{} {}", dch, header));
                for item in items {
                    let obj = item
                        .as_object()
                        .expect("tabular detection guaranteed object");
                    let mut cells: Vec<String> = Vec::with_capacity(keys.len());
                    for k in &keys {
                        let v = obj.get(k).unwrap();
                        let cell = match v {
                            Value::Null => primitives::format_null().to_string(),
                            Value::Bool(b) => primitives::format_bool(*b).to_string(),
                            Value::Number(n) => n.to_string(),
                            Value::String(s) => primitives::format_string(s, opts.delimiter),
                            _ => "null".to_string(),
                        };
                        cells.push(cell);
                    }
                    let row = join_with_delim(&cells, dch);
                    w.line_list_item(indent, &row);
                }
            } else {
                // Fallback to list form
                if items.is_empty() {
                    // Empty array at any level: use [0]: syntax per spec
                    w.line(indent, "[0]:");
                } else {
                    for item in items {
                        match item {
                            Value::Null => w.line_list_item(indent, primitives::format_null()),
                            Value::Bool(b) => w.line_list_item(indent, primitives::format_bool(*b)),
                            Value::Number(n) => w.line_list_item(indent, &n.to_string()),
                            Value::String(s) => {
                                w.line_list_item(indent, &primitives::format_string(s, opts.delimiter))
                            }
                            Value::Array(_) | Value::Object(_) => {
                                // Start list item then nested block
                                w.line(indent, "-");
                                encode_value(item, w, opts, indent + 2)?;
                            }
                        }
                    }
                }
            }
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                // Empty object at any level: use {0}: syntax (symmetrical with [0]:)
                w.line(indent, "{0}:");
            } else {
                for (k, v) in obj {
                    let key = primitives::format_string(k, opts.delimiter);
                    match v {
                        Value::Null => w.line_kv(indent, &key, primitives::format_null()),
                        Value::Bool(b) => w.line_kv(indent, &key, primitives::format_bool(*b)),
                        Value::Number(n) => w.line_kv(indent, &key, &n.to_string()),
                        Value::String(s) => {
                            w.line_kv(indent, &key, &primitives::format_string(s, opts.delimiter))
                        }
                        Value::Array(_) | Value::Object(_) => {
                            w.line_key_only(indent, &key);
                            encode_value(v, w, opts, indent + 2)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(feature = "serde")]
pub fn is_tabular_array(arr: &[Value]) -> Option<Vec<String>> {
    if arr.is_empty() {
        return None;
    }
    let mut keys: Option<Vec<String>> = None;
    for v in arr {
        let obj = match v {
            Value::Object(m) => m,
            _ => return None,
        };
        let kset: Vec<String> = obj.keys().cloned().collect();

        if let Some(ref ks) = keys {
            // Check if keys match (order-insensitive comparison)
            let mut kset_sorted = kset.clone();
            kset_sorted.sort();
            let mut ks_sorted = ks.clone();
            ks_sorted.sort();
            if ks_sorted != kset_sorted {
                return None;
            }
        } else {
            // Use the key order from the first object
            keys = Some(kset);
        }
        // All values must be primitives
        for (_, vv) in obj.iter() {
            match vv {
                Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
                _ => return None,
            }
        }
    }
    keys
}

fn join_with_delim(cells: &[String], dch: char) -> String {
    if dch == '\t' {
        cells.join("\t")
    } else {
        cells.join(&format!("{} ", dch))
    }
}
