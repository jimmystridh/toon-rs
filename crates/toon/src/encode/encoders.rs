use crate::{
    Result,
    encode::{primitives, writer::LineWriter},
    options::{Delimiter, Options},
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
        Value::String(s) => w.line_formatted(indent, s, opts.delimiter),
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
                let mut cells: Vec<String> = vec![String::new(); keys.len()];
                let mut row_buf = String::new();
                for item in items {
                    let obj = item
                        .as_object()
                        .expect("tabular detection guaranteed object");
                    for (idx, k) in keys.iter().enumerate() {
                        let v = obj.get(k).unwrap();
                        format_tabular_cell(&mut cells[idx], v, opts.delimiter);
                    }
                    join_cells_into(&cells, dch, &mut row_buf);
                    w.line_list_item(indent, &row_buf);
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
                                w.line_list_item_formatted(indent, s, opts.delimiter)
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
                    match v {
                        Value::Null => w.line_kv_key_formatted_raw(
                            indent,
                            k,
                            primitives::format_null(),
                            opts.delimiter,
                        ),
                        Value::Bool(b) => w.line_kv_key_formatted_raw(
                            indent,
                            k,
                            primitives::format_bool(*b),
                            opts.delimiter,
                        ),
                        Value::Number(n) => {
                            let num = n.to_string();
                            w.line_kv_key_formatted_raw(indent, k, &num, opts.delimiter);
                        }
                        Value::String(s) => w.line_kv_formatted(indent, k, s, opts.delimiter),
                        Value::Array(_) | Value::Object(_) => {
                            w.line_key_only_formatted(indent, k, opts.delimiter);
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

fn join_cells_into(cells: &[String], dch: char, out: &mut String) {
    out.clear();
    for (idx, cell) in cells.iter().enumerate() {
        if idx > 0 {
            if dch == '\t' {
                out.push('\t');
            } else {
                out.push(dch);
                out.push(' ');
            }
        }
        out.push_str(cell);
    }
}

fn format_tabular_cell(buf: &mut String, value: &Value, delim: Delimiter) {
    buf.clear();
    match value {
        Value::Null => buf.push_str(primitives::format_null()),
        Value::Bool(b) => buf.push_str(primitives::format_bool(*b)),
        Value::Number(n) => buf.push_str(&n.to_string()),
        Value::String(s) => {
            if primitives::needs_quotes(s, delim) {
                primitives::escape_and_quote_into(buf, s);
            } else {
                buf.push_str(s);
            }
        }
        _ => buf.push_str(primitives::format_null()),
    }
}
