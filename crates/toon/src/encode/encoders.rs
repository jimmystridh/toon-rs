use crate::{
    Result,
    encode::{primitives, writer::LineWriter},
    options::{KeyFolding, Options},
};

#[cfg(feature = "serde")]
use serde_json::Value;

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeSet, format, string::String, vec::Vec};

/// Check if an array of primitives can be emitted inline (no nested arrays/objects)
fn is_primitive_array(items: &[Value]) -> bool {
    items.iter().all(|v| {
        matches!(
            v,
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
        )
    })
}

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
            encode_array(items, w, opts, indent)?;
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                // Empty object: just key: with nothing following
                // (when nested, the parent emits "key:" and this produces no additional lines)
            } else {
                for (k, v) in obj {
                    encode_object_field_internal(k, v, w, opts, indent)?;
                }
            }
        }
    }
    Ok(())
}

/// Encode an object field (key-value pair) - internal version without sibling context
fn encode_object_field_internal(
    key: &str,
    value: &Value,
    w: &mut LineWriter,
    opts: &Options,
    indent: usize,
) -> Result<()> {
    encode_object_field_with_context(key, value, w, opts, indent, None)
}

/// Encode an object field (key-value pair) - public API for streaming serializer
pub fn encode_object_field(
    key: &str,
    value: &Value,
    w: &mut LineWriter,
    opts: &Options,
    indent: usize,
) -> Result<()> {
    encode_object_field_with_context(key, value, w, opts, indent, None)
}

/// Encode an object field with sibling key context for collision detection
/// sibling_keys should contain all keys at the current object level
pub fn encode_object_field_with_siblings(
    key: &str,
    value: &Value,
    w: &mut LineWriter,
    opts: &Options,
    indent: usize,
    sibling_keys: &[String],
) -> Result<()> {
    encode_object_field_with_context(key, value, w, opts, indent, Some(sibling_keys))
}

/// Encode object field with optional sibling key context for collision detection
fn encode_object_field_with_context(
    key: &str,
    value: &Value,
    w: &mut LineWriter,
    opts: &Options,
    indent: usize,
    sibling_keys: Option<&[String]>,
) -> Result<()> {
    let key_fmt = primitives::format_key(key);

    // Track whether we should disable nested folding
    let mut disable_nested_folding = false;

    // Try key folding for object values
    if opts.key_folding == KeyFolding::Safe {
        if let Value::Object(obj) = value {
            // First check if folding is possible without sibling collision
            let possible_fold = try_fold_keys_no_collision(key, obj, opts);

            if let Some((folded_key, final_value)) = possible_fold {
                // Check if there's a sibling collision
                let has_collision = sibling_keys
                    .map(|s| s.contains(&folded_key))
                    .unwrap_or(false);

                if !has_collision {
                    // No collision - encode with the folded key
                    return encode_folded_value(&folded_key, final_value, w, opts, indent);
                } else {
                    // Collision detected - disable nested folding
                    disable_nested_folding = true;
                }
            }
        }
    }

    // Determine options for nested content
    let nested_opts = if disable_nested_folding {
        Options {
            key_folding: KeyFolding::Off,
            ..opts.clone()
        }
    } else {
        opts.clone()
    };

    // Standard encoding (no folding)
    match value {
        Value::Null => w.line_kv(indent, &key_fmt, primitives::format_null()),
        Value::Bool(b) => w.line_kv(indent, &key_fmt, primitives::format_bool(*b)),
        Value::Number(n) => w.line_kv(indent, &key_fmt, &n.to_string()),
        Value::String(s) => w.line_kv(
            indent,
            &key_fmt,
            &primitives::format_string(s, opts.delimiter),
        ),
        Value::Array(items) => {
            encode_keyed_array(&key_fmt, items, w, &nested_opts, indent)?;
        }
        Value::Object(obj) => {
            w.line_key_only(indent, &key_fmt);
            if !obj.is_empty() {
                let sibling_keys: Vec<String> = obj.keys().cloned().collect();
                for (k, v) in obj {
                    encode_object_field_with_context(
                        k,
                        v,
                        w,
                        &nested_opts,
                        indent + opts.indent,
                        Some(&sibling_keys),
                    )?;
                }
            }
        }
    }
    Ok(())
}

/// Try to fold keys without checking sibling collision (for collision detection)
fn try_fold_keys_no_collision<'a>(
    initial_key: &str,
    initial_obj: &'a serde_json::Map<String, Value>,
    opts: &Options,
) -> Option<(String, &'a Value)> {
    try_fold_keys(initial_key, initial_obj, opts, None)
}

/// Try to fold a chain of single-key objects into a dotted path
/// Returns Some((folded_key, final_value)) if folding succeeds
///
/// flattenDepth controls the maximum number of segments in the folded key:
/// - flattenDepth=0: No folding
/// - flattenDepth=1: No practical effect (can't have less than 2 segments)
/// - flattenDepth=2: Allows 2-segment paths like "a.b"
/// - flattenDepth=N: Allows up to N segments
/// - None (Infinity): No limit
fn try_fold_keys<'a>(
    initial_key: &str,
    initial_obj: &'a serde_json::Map<String, Value>,
    opts: &Options,
    sibling_keys: Option<&[String]>,
) -> Option<(String, &'a Value)> {
    // flattenDepth is the max number of segments allowed in the folded key
    let max_segments = opts.flatten_depth.unwrap_or(usize::MAX);

    // Need at least 2 segments to fold (e.g., "a.b")
    // So if max_segments < 2, no folding is possible
    if max_segments < 2 {
        return None;
    }

    // Must be single-key to start folding
    if initial_obj.len() != 1 {
        return None;
    }

    // Check if initial key needs quotes (safe mode skips folding)
    if key_needs_quotes(initial_key) {
        return None;
    }

    // Collect the chain of single-key objects
    let mut path_segments = vec![initial_key.to_string()];
    let mut current_obj = initial_obj;
    let mut final_value: Option<&Value> = None;

    // Walk down the chain
    loop {
        // Get the single key-value pair
        let (k, v) = current_obj.iter().next().unwrap();

        // Check if this key needs quotes
        if key_needs_quotes(k) {
            // Can't add this key - we stop at the current position
            // The value we return is the single-key object itself (current_obj as Value)
            // Actually, we need to return the object containing k, not v
            break;
        }

        path_segments.push(k.clone());

        // Check if we've reached the depth limit
        if path_segments.len() >= max_segments {
            // We've hit the limit - return v as the final value
            final_value = Some(v);
            break;
        }

        match v {
            Value::Object(inner) if inner.len() == 1 => {
                // Continue folding
                current_obj = inner;
            }
            _ => {
                // End of chain - v is our final value
                final_value = Some(v);
                break;
            }
        }
    }

    // Only return if we have at least 2 segments (a proper fold)
    if path_segments.len() >= 2 {
        // If we have a final value from the loop, use it
        if let Some(fv) = final_value {
            let folded_key = path_segments.join(".");
            if let Some(siblings) = sibling_keys {
                if siblings.contains(&folded_key) {
                    return None;
                }
            }
            return Some((folded_key, fv));
        }

        // We broke out due to quoted key - need to find the value at the last position
        // Navigate to get the value at path_segments[1..]
        let mut nav_obj = initial_obj;
        for seg in path_segments.iter().skip(1).take(path_segments.len() - 2) {
            if let Some(Value::Object(inner)) = nav_obj.get(seg) {
                nav_obj = inner;
            } else {
                return None;
            }
        }

        // Get the value at the last segment
        let last_seg = &path_segments[path_segments.len() - 1];
        if let Some(val) = nav_obj.get(last_seg) {
            let folded_key = path_segments.join(".");
            if let Some(siblings) = sibling_keys {
                if siblings.contains(&folded_key) {
                    return None;
                }
            }
            return Some((folded_key, val));
        }
    }

    None
}

/// Check if a key requires quotes (and thus shouldn't be folded in safe mode)
fn key_needs_quotes(key: &str) -> bool {
    // A key needs quotes if it's not a valid IdentifierSegment
    // IdentifierSegment: [A-Za-z_][A-Za-z0-9_]*
    if key.is_empty() {
        return true;
    }
    let mut chars = key.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return true;
    }
    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return true;
        }
    }
    false
}

/// Encode a folded key-value pair
/// After a fold is performed, we disable further folding in nested content
fn encode_folded_value(
    folded_key: &str,
    value: &Value,
    w: &mut LineWriter,
    opts: &Options,
    indent: usize,
) -> Result<()> {
    // Disable further folding for nested content
    let nested_opts = Options {
        key_folding: KeyFolding::Off,
        ..opts.clone()
    };

    match value {
        Value::Null => w.line_kv(indent, folded_key, primitives::format_null()),
        Value::Bool(b) => w.line_kv(indent, folded_key, primitives::format_bool(*b)),
        Value::Number(n) => w.line_kv(indent, folded_key, &n.to_string()),
        Value::String(s) => w.line_kv(
            indent,
            folded_key,
            &primitives::format_string(s, opts.delimiter),
        ),
        Value::Array(items) => {
            encode_keyed_array(folded_key, items, w, &nested_opts, indent)?;
        }
        Value::Object(obj) => {
            w.line_key_only(indent, folded_key);
            if !obj.is_empty() {
                let sibling_keys: Vec<String> = obj.keys().cloned().collect();
                for (k, v) in obj {
                    encode_object_field_with_context(
                        k,
                        v,
                        w,
                        &nested_opts,
                        indent + opts.indent,
                        Some(&sibling_keys),
                    )?;
                }
            }
        }
    }
    Ok(())
}

/// Encode a keyed array (array as an object field): `key[N]{fields}: ...` or `key[N]: ...`
fn encode_keyed_array(
    key: &str,
    items: &[Value],
    w: &mut LineWriter,
    opts: &Options,
    indent: usize,
) -> Result<()> {
    let len = items.len();
    let delim = opts.delimiter;
    let dch = primitives::delimiter_char(delim);

    if items.is_empty() {
        // Empty array: key[0]:
        w.line(
            indent,
            &format!("{}{}:", key, primitives::format_bracket_segment(0, delim)),
        );
        return Ok(());
    }

    // Check for tabular array (uniform objects with primitive values)
    if let Some(keys) = is_tabular_array(items) {
        // Tabular: key[N]{f1,f2,...}:
        let field_cells: Vec<String> = keys.iter().map(|k| primitives::format_key(k)).collect();
        let header = format!(
            "{}{}",
            key,
            primitives::format_tabular_header(len, &field_cells, delim)
        );
        w.line(indent, &header);

        // Emit rows at indent+2
        for item in items {
            let obj = item
                .as_object()
                .expect("tabular detection guaranteed object");
            let cells: Vec<String> = keys
                .iter()
                .map(|k| {
                    let v = obj.get(k).unwrap();
                    format_primitive_value(v, delim)
                })
                .collect();
            let row = join_with_delim(&cells, dch);
            w.line(indent + opts.indent, &row);
        }
        return Ok(());
    }

    // Check for inline primitive array
    if is_primitive_array(items) {
        // Inline: key[N]: v1,v2,v3
        let values: Vec<String> = items
            .iter()
            .map(|v| format_primitive_value(v, delim))
            .collect();
        let inline = join_with_delim(&values, dch);
        w.line(
            indent,
            &format!(
                "{}{}: {}",
                key,
                primitives::format_bracket_segment(len, delim),
                inline
            ),
        );
        return Ok(());
    }

    // Mixed/complex array: key[N]: with list items
    w.line(
        indent,
        &format!(
            "{}{}",
            key,
            primitives::format_expanded_array_header(len, delim)
        ),
    );
    for item in items {
        encode_list_item(item, w, opts, indent + opts.indent)?;
    }
    Ok(())
}

/// Encode a root-level array (no key prefix)
fn encode_array(items: &[Value], w: &mut LineWriter, opts: &Options, indent: usize) -> Result<()> {
    let len = items.len();
    let delim = opts.delimiter;
    let dch = primitives::delimiter_char(delim);

    if items.is_empty() {
        // Empty root array: [0]:
        w.line(indent, &primitives::format_expanded_array_header(0, delim));
        return Ok(());
    }

    // Check for tabular array
    if let Some(keys) = is_tabular_array(items) {
        // Root tabular: [N]{f1,f2,...}:
        let field_cells: Vec<String> = keys.iter().map(|k| primitives::format_key(k)).collect();
        let header = primitives::format_tabular_header(len, &field_cells, delim);
        w.line(indent, &header);

        // Emit rows at indent+2
        for item in items {
            let obj = item
                .as_object()
                .expect("tabular detection guaranteed object");
            let cells: Vec<String> = keys
                .iter()
                .map(|k| {
                    let v = obj.get(k).unwrap();
                    format_primitive_value(v, delim)
                })
                .collect();
            let row = join_with_delim(&cells, dch);
            w.line(indent + opts.indent, &row);
        }
        return Ok(());
    }

    // Check for inline primitive array
    if is_primitive_array(items) {
        // Root inline: [N]: v1,v2,v3
        let values: Vec<String> = items
            .iter()
            .map(|v| format_primitive_value(v, delim))
            .collect();
        let inline = join_with_delim(&values, dch);
        w.line(
            indent,
            &format!(
                "{}: {}",
                primitives::format_bracket_segment(len, delim),
                inline
            ),
        );
        return Ok(());
    }

    // Mixed/complex root array: [N]: with list items
    w.line(
        indent,
        &primitives::format_expanded_array_header(len, delim),
    );
    for item in items {
        encode_list_item(item, w, opts, indent + opts.indent)?;
    }
    Ok(())
}

/// Encode a list item (- prefix)
fn encode_list_item(item: &Value, w: &mut LineWriter, opts: &Options, indent: usize) -> Result<()> {
    match item {
        Value::Null => w.line_list_item(indent, primitives::format_null()),
        Value::Bool(b) => w.line_list_item(indent, primitives::format_bool(*b)),
        Value::Number(n) => w.line_list_item(indent, &n.to_string()),
        Value::String(s) => w.line_list_item(indent, &primitives::format_string(s, opts.delimiter)),
        Value::Array(inner) => {
            // Array of arrays: - [M]: v1,v2 or - [M]: with nested
            encode_list_item_array(inner, w, opts, indent)?;
        }
        Value::Object(obj) => {
            // Object as list item - see §10
            encode_list_item_object(obj, w, opts, indent)?;
        }
    }
    Ok(())
}

/// Encode an array as a list item: `- [M]: v1,v2` or `- [M]:` with nested
fn encode_list_item_array(
    items: &[Value],
    w: &mut LineWriter,
    opts: &Options,
    indent: usize,
) -> Result<()> {
    let len = items.len();
    let delim = opts.delimiter;
    let dch = primitives::delimiter_char(delim);

    if items.is_empty() {
        w.line_list_item(
            indent,
            &format!("{}:", primitives::format_bracket_segment(0, delim)),
        );
        return Ok(());
    }

    if is_primitive_array(items) {
        // Inline: - [M]: v1,v2
        let values: Vec<String> = items
            .iter()
            .map(|v| format_primitive_value(v, delim))
            .collect();
        let inline = join_with_delim(&values, dch);
        w.line_list_item(
            indent,
            &format!(
                "{}: {}",
                primitives::format_bracket_segment(len, delim),
                inline
            ),
        );
    } else {
        // Complex: - [M]: with nested list items
        w.line_list_item(
            indent,
            &primitives::format_expanded_array_header(len, delim),
        );
        for inner_item in items {
            encode_list_item(inner_item, w, opts, indent + opts.indent)?;
        }
    }
    Ok(())
}

/// Encode an object as a list item per §10
fn encode_list_item_object(
    obj: &serde_json::Map<String, Value>,
    w: &mut LineWriter,
    opts: &Options,
    indent: usize,
) -> Result<()> {
    if obj.is_empty() {
        // Empty object: bare hyphen
        w.line(indent, "-");
        return Ok(());
    }

    let mut iter = obj.iter();
    let (first_key, first_value) = iter.next().unwrap();
    let first_key_fmt = primitives::format_key(first_key);

    // Check if first field is a tabular array (§10 special case)
    if let Value::Array(items) = first_value {
        if let Some(keys) = is_tabular_array(items) {
            // §10: - key[N]{fields}: on hyphen line, rows at depth+2, other fields at depth+1
            let delim = opts.delimiter;
            let dch = primitives::delimiter_char(delim);
            let field_cells: Vec<String> = keys.iter().map(|k| primitives::format_key(k)).collect();
            let header = format!(
                "{}{}",
                first_key_fmt,
                primitives::format_tabular_header(items.len(), &field_cells, delim)
            );
            w.line_list_item(indent, &header);

            // Rows at depth+2 (indent + 4 relative to list item indent)
            for item in items {
                let inner_obj = item.as_object().unwrap();
                let cells: Vec<String> = keys
                    .iter()
                    .map(|k| {
                        let v = inner_obj.get(k).unwrap();
                        format_primitive_value(v, delim)
                    })
                    .collect();
                let row = join_with_delim(&cells, dch);
                w.line(indent + 4, &row);
            }

            // Other fields at depth+1 (indent + opts.indent)
            for (k, v) in iter {
                encode_object_field(k, v, w, opts, indent + opts.indent)?;
            }
            return Ok(());
        }
    }

    // Standard case: first field on hyphen line
    match first_value {
        Value::Null => w.line(
            indent,
            &format!("- {}: {}", first_key_fmt, primitives::format_null()),
        ),
        Value::Bool(b) => w.line(
            indent,
            &format!("- {}: {}", first_key_fmt, primitives::format_bool(*b)),
        ),
        Value::Number(n) => w.line(indent, &format!("- {}: {}", first_key_fmt, n)),
        Value::String(s) => {
            let v = primitives::format_string(s, opts.delimiter);
            w.line(indent, &format!("- {}: {}", first_key_fmt, v));
        }
        Value::Array(items) => {
            // Non-tabular array as first field
            let len = items.len();
            let delim = opts.delimiter;
            if items.is_empty() {
                w.line(
                    indent,
                    &format!(
                        "- {}{}:",
                        first_key_fmt,
                        primitives::format_bracket_segment(0, delim)
                    ),
                );
            } else if is_primitive_array(items) {
                let dch = primitives::delimiter_char(delim);
                let values: Vec<String> = items
                    .iter()
                    .map(|v| format_primitive_value(v, delim))
                    .collect();
                let inline = join_with_delim(&values, dch);
                w.line(
                    indent,
                    &format!(
                        "- {}{}: {}",
                        first_key_fmt,
                        primitives::format_bracket_segment(len, delim),
                        inline
                    ),
                );
            } else {
                w.line(
                    indent,
                    &format!(
                        "- {}{}",
                        first_key_fmt,
                        primitives::format_expanded_array_header(len, delim)
                    ),
                );
                for inner_item in items {
                    encode_list_item(inner_item, w, opts, indent + 4)?;
                }
            }
        }
        Value::Object(inner_obj) => {
            w.line(indent, &format!("- {}:", first_key_fmt));
            if !inner_obj.is_empty() {
                for (k, v) in inner_obj {
                    encode_object_field(k, v, w, opts, indent + 4)?;
                }
            }
        }
    }

    // Remaining fields at depth+1 (indent + opts.indent)
    for (k, v) in iter {
        encode_object_field(k, v, w, opts, indent + opts.indent)?;
    }
    Ok(())
}

/// Format a primitive value for inline/tabular output
fn format_primitive_value(v: &Value, delim: crate::options::Delimiter) -> String {
    match v {
        Value::Null => primitives::format_null().to_string(),
        Value::Bool(b) => primitives::format_bool(*b).to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => primitives::format_string(s, delim),
        _ => "null".to_string(),
    }
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
    cells.join(&dch.to_string())
}
