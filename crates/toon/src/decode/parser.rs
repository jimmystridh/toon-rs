use crate::decode::scanner::{ParsedLine, LineKind, scan};

#[cfg(not(feature = "std"))]
use alloc::{format, string::{String, ToString}, vec::Vec};

use crate::value::{Value, Number};

pub struct Parser {
    lines: Vec<ParsedLine>,
    idx: usize,
    strict: bool,
    error: Option<crate::error::Error>,
}

impl Parser {
    pub fn from_str(input: &str) -> Self {
        Self { lines: scan(input), idx: 0, strict: false, error: None }
    }

    pub fn from_str_with_strict(input: &str, strict: bool) -> Self {
        Self { lines: scan(input), idx: 0, strict, error: None }
    }

    pub fn is_empty(&self) -> bool { self.lines.is_empty() }

    fn skip_blanks(&mut self) {
        while let Some(line) = self.lines.get(self.idx) {
            if matches!(line.kind, LineKind::Blank) { self.idx += 1; } else { break; }
        }
    }

    fn peek(&self) -> Option<&ParsedLine> { self.lines.get(self.idx) }

    fn next(&mut self) -> Option<&ParsedLine> { let i = self.idx; self.idx += 1; self.lines.get(i) }

    fn parse_scalar_token(&self, s: &str) -> Value {
        if s.starts_with('"') {
            if let Some(st) = unescape_json_string(s) { return Value::String(st); }
        }
        match s {
            "true" => return Value::Bool(true),
            "false" => return Value::Bool(false),
            "null" => return Value::Null,
            _ => {}
        }
        if let Ok(i) = s.parse::<i64>() { return Value::Number(Number::I64(i)); }
        if let Ok(u) = s.parse::<u64>() { return Value::Number(Number::U64(u)); }
        if let Ok(f) = s.parse::<f64>() { return Value::Number(Number::F64(f)); }
        Value::String(s.to_string())
    }

    fn parse_key_token(&self, k: &str) -> String {
        if k.starts_with('"') {
            if let Some(st) = unescape_json_string(k) { return st; }
        }
        k.to_string()
    }

    fn parse_array(&mut self, indent: usize) -> Value {
        let mut arr = Vec::new();
        loop {
            self.skip_blanks();
            let kind = {
                let Some(line) = self.peek() else { break; };
                if line.indent != indent { break; }
                line.kind.clone()
            };
            match kind {
                LineKind::ListItem { value } => {
                    self.next();
                    if let Some(vs) = value {
                        arr.push(self.parse_scalar_token(&vs));
                    } else {
                        // Nested block
                        let child_indent = indent + 2;
                        arr.push(self.parse_node(child_indent));
                    }
                }
                _ => break,
            }
        }
        Value::Array(arr)
    }

    fn parse_object(&mut self, indent: usize) -> Value {
        let mut map: Vec<(String, Value)> = Vec::new();
        loop {
            self.skip_blanks();
            let kind = {
                let Some(line) = self.peek() else { break; };
                if line.indent != indent { break; }
                line.kind.clone()
            };
            match kind {
                LineKind::KeyValue { key, value } => {
                    self.next();
                    let k = self.parse_key_token(&key);
                    let v = self.parse_scalar_token(&value);
                    map.push((k, v));
                }
                LineKind::KeyOnly { key } => {
                    self.next();
                    let k = self.parse_key_token(&key);
                    let child_indent = indent + 2;
                    // Check for tabular header line
                    let mut handled = false;
                    if let Some(nl) = self.peek() {
                        if nl.indent == child_indent {
                            let kind = nl.kind.clone();
                            if let LineKind::Scalar(s) = kind {
                                if let Some((dch, header_str)) = parse_header(&s) {
                                    // Strict: delimiter must be one of allowed
                                    if self.strict && !(dch == ',' || dch == '\t' || dch == '|') {
                                        let line_no = self.idx + 1;
                                        self.error = Some(crate::error::Error::Syntax { line: line_no, message: format!("invalid header delimiter '{}': expected ',', '\\t', or '|'", dch) });
                                    }
                                    self.next(); // consume header line
                                    let raw_header_tokens = split_delim_aware(header_str, dch);
                                    let header_keys = raw_header_tokens.iter().map(|h| self.parse_key_token(h)).collect::<Vec<_>>();
                                    // Strict: header must be non-empty and unique keys, and tokens requiring quotes must be quoted
if self.strict {
                                        if header_keys.is_empty() {
                                            let line_no = self.idx; // just consumed header
                                            self.error = Some(crate::error::Error::Syntax { line: line_no, message: "empty tabular header".to_string() });
                                        }
                                        for (_i, htok) in raw_header_tokens.iter().enumerate() {
                                            if !is_quoted_token(htok) && token_requires_quotes(htok, dch) {
                                                let line_no = self.idx; // header line
                                                self.error = Some(crate::error::Error::Syntax { line: line_no, message: format!("unquoted header token requires quotes: {}", htok) });
                                                break;
                                            }
                                        }
                                        for i in 0..header_keys.len() {
                                            for j in (i+1)..header_keys.len() {
                                                if header_keys[i] == header_keys[j] {
                                                    let line_no = self.idx; // header line
                                                    self.error = Some(crate::error::Error::Syntax { line: line_no, message: format!("duplicate header key: {}", header_keys[i]) });
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    let expected_cells = header_keys.len();
                                    let mut rows: Vec<Value> = Vec::new();
                                    loop {
                                        // strict: no blank lines inside table block
                                        if self.strict {
                                            if let Some(bl) = self.peek() {
                                                if matches!(bl.kind, LineKind::Blank) {
                                                    let line_no = self.idx + 1;
                                                    self.error = Some(crate::error::Error::Syntax { line: line_no, message: "blank line inside table".to_string() });
                                                }
                                            }
                                        }
                                        self.skip_blanks();
                                        let kind2 = {
                                            let Some(rowl) = self.peek() else { break; };
                                            if rowl.indent != child_indent { break; }
                                            rowl.kind.clone()
                                        };
                                        match kind2 {
                                            LineKind::ListItem { value: Some(rs) } => {
                                                let row_line = self.idx + 1;
                                                self.next();
                                                let row_trimmed = rs.trim_end();
                                                if self.strict && row_trimmed.ends_with(dch) {
                                                    self.error = Some(crate::error::Error::Syntax { line: row_line, message: "trailing delimiter in row".to_string() });
                                                }
                                                let cells = split_delim_aware(&rs, dch);
                                                if self.strict && self.error.is_none() && cells.len() != expected_cells {
                                                    self.error = Some(crate::error::Error::Syntax { line: row_line, message: format!("row cell count {} does not match header {}", cells.len(), expected_cells) });
                                                }
                                                if self.strict && self.error.is_none() {
                                                    for ctok in &cells {
                                                        if !is_quoted_token(ctok) && cell_token_requires_quotes(ctok, dch) {
                                                            self.error = Some(crate::error::Error::Syntax { line: row_line, message: format!("unquoted cell requires quotes: {}", ctok) });
                                                            break;
                                                        }
                                                    }
                                                }
                                                let mut om: Vec<(String, Value)> = Vec::new();
                                                for (i, hk) in header_keys.iter().enumerate() {
                                                    let cell = cells.get(i).map(|s| s.as_str()).unwrap_or("null");
                                                    om.push((hk.clone(), self.parse_scalar_token(cell)));
                                                }
                                                rows.push(Value::Object(om));
                                            }
                                            _ => break,
                                        }
                                    }
                                    if self.strict && rows.is_empty() {
                                        let line_no = self.idx; // after header
                                        self.error = Some(crate::error::Error::Syntax { line: line_no, message: "empty table (no rows)".to_string() });
                                    }
                                    map.push((k.clone(), Value::Array(rows)));
                                    handled = true;
                                }
                            }
                        }
                    }
                    if !handled {
                        let v = self.parse_node(child_indent);
                        map.push((k, v));
                    }
                }
                _ => break,
            }
        }
        Value::Object(map)
    }

    fn parse_scalar_line(&mut self, _indent: usize) -> Value {
        let line = self.next().expect("expected scalar line");
        if let LineKind::Scalar(s) = &line.kind {
            let owned = s.clone();
            return self.parse_scalar_token(&owned);
        }
        Value::Null
    }

    fn parse_node(&mut self, indent: usize) -> Value {
        self.skip_blanks();
        let Some(line) = self.peek() else { return Value::Null; };
        match &line.kind {
            LineKind::ListItem { .. } if line.indent == indent => self.parse_array(indent),
            LineKind::KeyValue { .. } | LineKind::KeyOnly { .. } if line.indent == indent => self.parse_object(indent),
            LineKind::Scalar(_) if line.indent == indent => self.parse_scalar_line(indent),
            _ => {
                // If indentation doesn't match, return null
                Value::Null
            }
        }
    }

    pub fn parse_document(&mut self) -> Value {
        self.skip_blanks();
        if self.peek().is_none() { return Value::Null; }
        let indent = self.peek().unwrap().indent;
        self.parse_node(indent)
    }
}

fn parse_header(s: &str) -> Option<(char, &str)> {
    let mut it = s.chars();
    let at = it.next()?;
    if at != '@' { return None; }
    let dch = it.next()?;
    let rest = &s[2..];
    Some((dch, rest.trim_start()))
}

fn split_delim_aware(s: &str, dch: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_str = false;
    let mut escape = false;
    for ch in s.chars() {
        if escape { cur.push(ch); escape = false; continue; }
        match ch {
            '\\' if in_str => { cur.push(ch); escape = true; }
            '"' => { in_str = !in_str; cur.push(ch); }
            c if c == dch && !in_str => {
                out.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }
    if !cur.is_empty() { out.push(cur.trim().to_string()); }
    out
}

fn is_quoted_token(s: &str) -> bool {
    let t = s.trim();
    t.starts_with('"') && t.ends_with('"') && t.len() >= 2
}

fn token_requires_quotes(s: &str, dch: char) -> bool {
    use crate::options::Delimiter;
    let delim = match dch {
        ',' => Delimiter::Comma,
        '\t' => Delimiter::Tab,
        '|' => Delimiter::Pipe,
        _ => Delimiter::Comma,
    };
    crate::encode::primitives::needs_quotes(s, delim)
}

fn cell_token_requires_quotes(s: &str, dch: char) -> bool {
    // Stricter check for row cells that ignores numeric/boolean-like tokens,
    // focusing only on characters that would break parsing if left unquoted.
    if s.is_empty() { return true; }
    let t = s.trim();
    if t.is_empty() { return true; }
    // Delimiter presence would have split already for unquoted tokens, but keep for safety
    if t.contains(dch) { return true; }
    // Unquoted colon is ambiguous inside cells
    if t.contains(':') { return true; }
    // Dangerous characters requiring escaping in quoted strings
    if t.chars().any(|c| c == '"' || c == '\\' || is_control_char(c)) { return true; }
    // Leading/trailing spaces would be lost; enforce quoting
    if t.starts_with(' ') || t.ends_with(' ') { return true; }
    // A lone hyphen or strings starting with hyphen (non-numeric) can be ambiguous; require quotes
    if t == "-" { return true; }
    // If it starts with '-' but is not a valid number, require quotes
    if t.starts_with('-') && t.parse::<f64>().is_err() { return true; }
    false
}

fn is_control_char(c: char) -> bool {
    let u = c as u32;
    u < 0x20 || u == 0x7F
}

pub fn parse_to_internal_value(input: &str) -> Value {
    let mut p = Parser::from_str(input);
    p.parse_document()
}

pub fn parse_to_value_with_strict(input: &str, strict: bool) -> Result<Value, crate::error::Error> {
    let mut p = Parser::from_str_with_strict(input, strict);
    let v = p.parse_document();
    if let Some(err) = p.error { Err(err) } else { Ok(v) }
}

#[cfg(feature = "json")]
pub fn parse_to_value(input: &str) -> serde_json::Value {
    let v = parse_to_internal_value(input);
    to_json_value(v)
}

#[cfg(feature = "json")]
fn to_json_value(v: Value) -> serde_json::Value {
    match v {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(b),
        Value::Number(n) => match n {
            Number::I64(i) => serde_json::Value::Number(i.into()),
            Number::U64(u) => serde_json::Value::Number(u.into()),
            Number::F64(f) => serde_json::Number::from_f64(f)
                .map(serde_json::Value::Number)
                .unwrap_or_else(|| serde_json::Value::String(f.to_string())),
        },
        Value::String(s) => serde_json::Value::String(s),
        Value::Array(a) => serde_json::Value::Array(a.into_iter().map(to_json_value).collect()),
        Value::Object(pairs) => {
            let mut m = serde_json::Map::new();
            for (k, vv) in pairs { m.insert(k, to_json_value(vv)); }
            serde_json::Value::Object(m)
        }
    }
}

fn unescape_json_string(s: &str) -> Option<String> {
    // Expecting a JSON string literal like "..."
    if !s.starts_with('"') || !s.ends_with('"') || s.len() < 2 { return None; }
    let inner = &s[1..s.len()-1];
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next()? {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'b' => out.push('\u{0008}'),
                'f' => out.push('\u{000C}'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                'u' => {
                    let mut code = 0u32;
                    for _ in 0..4 {
                        let d = chars.next()?;
                        code = (code << 4) | hex_val(d)?;
                    }
                    if let Some(c) = core::char::from_u32(code) { out.push(c); } else { return None; }
                }
                _ => return None,
            }
        } else {
            out.push(ch);
        }
    }
    Some(out)
}

fn hex_val(c: char) -> Option<u32> {
    match c {
        '0'..='9' => Some((c as u32) - ('0' as u32)),
        'a'..='f' => Some(10 + (c as u32) - ('a' as u32)),
        'A'..='F' => Some(10 + (c as u32) - ('A' as u32)),
        _ => None,
    }
}
