use js_sys::{Array, Object, Reflect};
use toon::encode::{primitives, writer::LineWriter};
use toon::options::{Delimiter, Options};
use wasm_bindgen::{JsCast, JsValue};

pub fn encode_js_value_to_string(value: &JsValue, options: &Options) -> Result<String, JsValue> {
    let mut writer = LineWriter::new();
    encode_value(value, &mut writer, options, 0)?;
    Ok(writer.into_string())
}

fn encode_value(
    value: &JsValue,
    writer: &mut LineWriter,
    options: &Options,
    indent: usize,
) -> Result<(), JsValue> {
    if value.is_undefined() || value.is_null() {
        writer.line(indent, primitives::format_null());
        return Ok(());
    }

    if let Some(b) = value.as_bool() {
        writer.line(indent, primitives::format_bool(b));
        return Ok(());
    }

    if let Some(n) = value.as_f64() {
        write_number(writer, indent, n);
        return Ok(());
    }

    if let Some(s) = value.as_string() {
        writer.line_formatted(indent, &s, options.delimiter);
        return Ok(());
    }

    if Array::is_array(value) {
        let array = Array::from(value);
        encode_array(&array, writer, options, indent)
    } else if value.is_object() {
        let obj: &Object = value.unchecked_ref();
        encode_object(obj, writer, options, indent)
    } else {
        Err(JsValue::from_str("Unsupported JS value for TOON encoding"))
    }
}

fn encode_array(
    array: &Array,
    writer: &mut LineWriter,
    options: &Options,
    indent: usize,
) -> Result<(), JsValue> {
    let len = array.length() as usize;
    if len == 0 {
        writer.line(indent, "[0]:");
        return Ok(());
    }

    if let Some(keys) = detect_tabular_array(array)? {
        emit_tabular(array, &keys, writer, options, indent)
    } else {
        for idx in 0..len as u32 {
            let item = array.get(idx);
            if item.is_undefined() || item.is_null() {
                writer.line_list_item(indent, primitives::format_null());
            } else if let Some(b) = item.as_bool() {
                writer.line_list_item(indent, primitives::format_bool(b));
            } else if let Some(n) = item.as_f64() {
                let num = format_number(n);
                writer.line_list_item(indent, &num);
            } else if let Some(s) = item.as_string() {
                writer.line_list_item_formatted(indent, &s, options.delimiter);
            } else {
                writer.line(indent, "-");
                encode_value(&item, writer, options, indent + 2)?;
            }
        }
        Ok(())
    }
}

fn emit_tabular(
    array: &Array,
    keys: &[String],
    writer: &mut LineWriter,
    options: &Options,
    indent: usize,
) -> Result<(), JsValue> {
    let dch = primitives::delimiter_char(options.delimiter);
    let header_cells: Vec<String> = keys
        .iter()
        .map(|k| primitives::format_string(k, options.delimiter))
        .collect();
    let header = join_with_delim(&header_cells, dch);
    writer.line(indent, &format!("@{} {}", dch, header));

    let len = array.length() as u32;
    let mut cells: Vec<String> = vec![String::new(); keys.len()];
    let mut row_buf = String::new();
    for idx in 0..len {
        let item = array.get(idx);
        let obj: &Object = item
            .dyn_ref()
            .ok_or_else(|| JsValue::from_str("Tabular rows must be objects"))?;
        for (cell_buf, key) in cells.iter_mut().zip(keys.iter()) {
            let key_val = JsValue::from(key.as_str());
            let value = Reflect::get(obj, &key_val)?;
            format_js_tabular_cell(cell_buf, &value, options.delimiter);
        }
        join_cells_into(&cells, dch, &mut row_buf);
        writer.line_list_item(indent, &row_buf);
    }
    Ok(())
}

fn detect_tabular_array(array: &Array) -> Result<Option<Vec<String>>, JsValue> {
    let len = array.length() as u32;
    if len == 0 {
        return Ok(None);
    }
    let mut keys: Option<Vec<String>> = None;
    for idx in 0..len {
        let item = array.get(idx);
        let obj: &Object = match item.dyn_ref() {
            Some(o) => o,
            None => return Ok(None),
        };
        let obj_keys_array = Object::keys(obj);
        let mut obj_keys: Vec<String> = Vec::with_capacity(obj_keys_array.length() as usize);
        for j in 0..obj_keys_array.length() as u32 {
            let key = obj_keys_array.get(j);
            let key_str = key
                .as_string()
                .ok_or_else(|| JsValue::from_str("Object keys must be strings"))?;
            obj_keys.push(key_str);
        }
        if let Some(existing) = &keys {
            if !same_key_set(existing, &obj_keys) {
                return Ok(None);
            }
        } else {
            keys = Some(obj_keys.clone());
        }
        for key_str in &obj_keys {
            let key_value = JsValue::from(key_str.as_str());
            let value = Reflect::get(obj, &key_value)?;
            if !is_primitive(&value) {
                return Ok(None);
            }
        }
    }
    Ok(keys)
}

fn same_key_set(a: &[String], b: &[String]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut sa = a.to_vec();
    let mut sb = b.to_vec();
    sa.sort();
    sb.sort();
    sa == sb
}

fn encode_object(
    obj: &Object,
    writer: &mut LineWriter,
    options: &Options,
    indent: usize,
) -> Result<(), JsValue> {
    let keys = Object::keys(obj);
    if keys.length() == 0 {
        writer.line(indent, "{0}:");
        return Ok(());
    }
    let mut key_buf = String::new();
    let mut string_buf = String::new();

    for idx in 0..keys.length() as u32 {
        let key = keys.get(idx);
        key_buf.clear();
        let key_str = key
            .as_string()
            .ok_or_else(|| JsValue::from_str("Object keys must be strings"))?;
        key_buf.push_str(&key_str);
        let value = Reflect::get(obj, &key)?;
        if value.is_undefined() || value.is_null() {
            writer.line_kv_key_formatted_raw(
                indent,
                &key_buf,
                primitives::format_null(),
                options.delimiter,
            );
        } else if let Some(b) = value.as_bool() {
            writer.line_kv_key_formatted_raw(
                indent,
                &key_buf,
                primitives::format_bool(b),
                options.delimiter,
            );
        } else if let Some(n) = value.as_f64() {
            let num = format_number(n);
            writer.line_kv_key_formatted_raw(indent, &key_buf, &num, options.delimiter);
        } else if let Some(s) = value.as_string() {
            string_buf.clear();
            string_buf.push_str(&s);
            writer.line_kv_formatted(indent, &key_buf, &string_buf, options.delimiter);
        } else {
            writer.line_key_only_formatted(indent, &key_buf, options.delimiter);
            encode_value(&value, writer, options, indent + 2)?;
        }
    }
    Ok(())
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 {
        if n >= i64::MIN as f64 && n <= i64::MAX as f64 {
            return (n as i64).to_string();
        }
        if n >= 0.0 && n <= u64::MAX as f64 {
            return (n as u64).to_string();
        }
    }
    primitives::format_f64(n)
}

fn is_primitive(value: &JsValue) -> bool {
    value.is_undefined()
        || value.is_null()
        || value.as_bool().is_some()
        || value.as_f64().is_some()
        || value.as_string().is_some()
}

fn format_js_tabular_cell(buf: &mut String, value: &JsValue, delimiter: Delimiter) {
    buf.clear();
    if value.is_undefined() || value.is_null() {
        buf.push_str(primitives::format_null());
    } else if let Some(b) = value.as_bool() {
        buf.push_str(primitives::format_bool(b));
    } else if let Some(n) = value.as_f64() {
        buf.push_str(&format_number(n));
    } else if let Some(s) = value.as_string() {
        if primitives::needs_quotes(&s, delimiter) {
            primitives::escape_and_quote_into(buf, &s);
        } else {
            buf.push_str(&s);
        }
    } else {
        buf.push_str(primitives::format_null());
    }
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

fn write_number(writer: &mut LineWriter, indent: usize, n: f64) {
    let formatted = format_number(n);
    writer.line(indent, &formatted);
}
