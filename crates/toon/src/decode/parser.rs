use crate::decode::scanner::{LineKind, ParsedLine, scan};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use crate::number::has_forbidden_leading_zeros;
use crate::value::{Number, Value};

pub struct Parser<'a> {
    lines: Vec<ParsedLine<'a>>,
    idx: usize,
    strict: bool,
    error: Option<crate::error::Error>,
}

impl<'a> Parser<'a> {
    pub fn from_input(input: &'a str) -> Self {
        Self {
            lines: scan(input),
            idx: 0,
            strict: false,
            error: None,
        }
    }

    pub fn from_input_with_strict(input: &'a str, strict: bool) -> Self {
        Self {
            lines: scan(input),
            idx: 0,
            strict,
            error: None,
        }
    }

    pub fn from_lines(lines: Vec<ParsedLine<'a>>, strict: bool) -> Self {
        Self {
            lines,
            idx: 0,
            strict,
            error: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    fn skip_blanks(&mut self) {
        while let Some(line) = self.lines.get(self.idx) {
            if matches!(line.kind, LineKind::Blank) {
                self.idx += 1;
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<&ParsedLine<'a>> {
        self.lines.get(self.idx)
    }

    fn next(&mut self) -> Option<&ParsedLine<'a>> {
        let i = self.idx;
        self.idx += 1;
        self.lines.get(i)
    }

    fn parse_scalar_token(&self, s: &str) -> Value {
        if s.starts_with('"') {
            if let Some(st) = unescape_json_string(s) {
                return Value::String(st);
            }
        }
        if has_forbidden_leading_zeros(s) {
            return Value::String(s.to_string());
        }
        match s {
            "true" => return Value::Bool(true),
            "false" => return Value::Bool(false),
            "null" => return Value::Null,
            _ => {}
        }
        // Fast path for pure integers (ASCII digits, optional leading '-')
        let bs = s.as_bytes();
        if !bs.is_empty() {
            if bs[0] == b'-' {
                if bs.len() > 1 && bs[1..].iter().all(|c| c.is_ascii_digit()) {
                    if let Ok(i) = s.parse::<i64>() {
                        return Value::Number(Number::I64(i));
                    }
                }
            } else if bs.iter().all(|c| c.is_ascii_digit()) {
                if let Ok(u) = s.parse::<u64>() {
                    return Value::Number(Number::U64(u));
                }
            }
        }
        match classify_numeric_hint(s) {
            Some(NumHint::Float) => {
                if let Ok(f) = s.parse::<f64>() {
                    return Value::Number(Number::F64(f));
                }
            }
            Some(NumHint::IntSigned) => {
                if let Ok(i) = s.parse::<i64>() {
                    return Value::Number(Number::I64(i));
                }
                if let Ok(f) = s.parse::<f64>() {
                    return Value::Number(Number::F64(f));
                }
            }
            Some(NumHint::IntUnsigned) => {
                if let Ok(u) = s.parse::<u64>() {
                    return Value::Number(Number::U64(u));
                }
                if let Ok(f) = s.parse::<f64>() {
                    return Value::Number(Number::F64(f));
                }
            }
            None => {}
        }
        Value::String(s.to_string())
    }

    fn parse_key_token(&self, k: &str) -> String {
        if k.starts_with('"') {
            if let Some(st) = unescape_json_string(k) {
                return st;
            }
        }
        k.to_string()
    }

    fn parse_array(&mut self, indent: usize) -> Value {
        let mut arr = Vec::new();
        loop {
            self.skip_blanks();
            let (take, item_val): (bool, Option<&str>) = if let Some(line) = self.peek() {
                if line.indent != indent {
                    break;
                }
                match &line.kind {
                    LineKind::ListItem { value } => (true, *value),
                    _ => (false, None),
                }
            } else {
                break;
            };
            if !take {
                break;
            }
            self.next();
            if let Some(vs) = item_val {
                arr.push(self.parse_scalar_token(vs));
            } else {
                let child_indent = indent + 2;
                arr.push(self.parse_node(child_indent));
            }
        }
        Value::Array(arr)
    }

    fn parse_object(&mut self, indent: usize) -> Value {
        let mut map: Vec<(String, Value)> = Vec::new();
        loop {
            self.skip_blanks();
            let next_kind = if let Some(line) = self.peek() {
                if line.indent != indent {
                    break;
                }
                match &line.kind {
                    LineKind::KeyValue { key, value } => Some((Some(*key), Some(*value))),
                    LineKind::KeyOnly { key } => Some((Some(*key), None)),
                    _ => None,
                }
            } else {
                break;
            };
            let Some((key_opt, val_opt)) = next_kind else {
                break;
            };
            match (key_opt, val_opt) {
                (Some(kref), Some(vref)) => {
                    self.next();
                    let k = self.parse_key_token(kref);
                    let v = self.parse_scalar_token(vref);
                    map.push((k, v));
                }
                (Some(kref), None) => {
                    self.next();
                    let k = self.parse_key_token(kref);
                    let child_indent = indent + 2;
                    let mut handled = false;
                    if let Some(nl) = self.peek() {
                        if nl.indent == child_indent {
                            if let LineKind::Scalar(s0) = &nl.kind {
                                let header_text = *s0;
                                if let Some((dch, header_str)) = parse_header(header_text) {
                                    if self.strict && !(dch == ',' || dch == '\t' || dch == '|') {
                                        let line_no = self.idx + 1;
                                        self.error = Some(crate::error::Error::Syntax {
                                            line: line_no,
                                            message: format!(
                                                "invalid header delimiter '{}': expected ',', '\\t', or '|'",
                                                dch
                                            ),
                                        });
                                    }
                                    self.next();
                                    let raw_header_tokens = split_delim_aware(header_str, dch);
                                    let header_keys = raw_header_tokens
                                        .iter()
                                        .map(|h| self.parse_key_token(h))
                                        .collect::<Vec<_>>();
                                    if self.strict {
                                        if header_keys.is_empty() {
                                            let line_no = self.idx;
                                            self.error = Some(crate::error::Error::Syntax {
                                                line: line_no,
                                                message: "empty tabular header".to_string(),
                                            });
                                        }
                                        for &htok in raw_header_tokens.iter() {
                                            if !is_quoted_token(htok)
                                                && token_requires_quotes(htok, dch)
                                            {
                                                let line_no = self.idx;
                                                self.error = Some(crate::error::Error::Syntax {
                                                    line: line_no,
                                                    message: format!(
                                                        "unquoted header token requires quotes: {}",
                                                        htok
                                                    ),
                                                });
                                                break;
                                            }
                                        }
                                        for i in 0..header_keys.len() {
                                            for j in (i + 1)..header_keys.len() {
                                                if header_keys[i] == header_keys[j] {
                                                    let line_no = self.idx;
                                                    self.error =
                                                        Some(crate::error::Error::Syntax {
                                                            line: line_no,
                                                            message: format!(
                                                                "duplicate header key: {}",
                                                                header_keys[i]
                                                            ),
                                                        });
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    let expected_cells = header_keys.len();
                                    let mut rows: Vec<Value> = Vec::new();
                                    loop {
                                        if self.strict {
                                            if let Some(bl) = self.peek() {
                                                if matches!(bl.kind, LineKind::Blank) {
                                                    let line_no = self.idx + 1;
                                                    self.error =
                                                        Some(crate::error::Error::Syntax {
                                                            line: line_no,
                                                            message: "blank line inside table"
                                                                .to_string(),
                                                        });
                                                }
                                            }
                                        }
                                        self.skip_blanks();
                                        let row_item: Option<&str> = if let Some(rowl) = self.peek()
                                        {
                                            if rowl.indent != child_indent {
                                                break;
                                            }
                                            match &rowl.kind {
                                                LineKind::ListItem { value: Some(rs) } => Some(*rs),
                                                _ => None,
                                            }
                                        } else {
                                            break;
                                        };
                                        let Some(rs) = row_item else {
                                            break;
                                        };
                                        let row_line = self.idx + 1;
                                        self.next();
                                        let row_trimmed = rs.trim_end();
                                        if self.strict
                                            && row_trimmed.as_bytes().last().copied()
                                                == Some(dch as u8)
                                        {
                                            self.error = Some(crate::error::Error::Syntax {
                                                line: row_line,
                                                message: "trailing delimiter in row".to_string(),
                                            });
                                        }
                                        let cells = split_delim_aware(rs, dch);
                                        if self.strict
                                            && self.error.is_none()
                                            && cells.len() != expected_cells
                                        {
                                            self.error = Some(crate::error::Error::Syntax {
                                                line: row_line,
                                                message: format!(
                                                    "row cell count {} does not match header {}",
                                                    cells.len(),
                                                    expected_cells
                                                ),
                                            });
                                        }
                                        if self.strict && self.error.is_none() {
                                            for ctok in &cells {
                                                if !is_quoted_token(ctok)
                                                    && cell_token_requires_quotes(ctok, dch)
                                                {
                                                    self.error =
                                                        Some(crate::error::Error::Syntax {
                                                            line: row_line,
                                                            message: format!(
                                                                "unquoted cell requires quotes: {}",
                                                                ctok
                                                            ),
                                                        });
                                                    break;
                                                }
                                            }
                                        }
                                        let mut om: Vec<(String, Value)> =
                                            Vec::with_capacity(expected_cells);
                                        for (i, hk) in header_keys.iter().enumerate() {
                                            let cell = cells.get(i).copied().unwrap_or("null");
                                            om.push((hk.clone(), self.parse_scalar_token(cell)));
                                        }
                                        rows.push(Value::Object(om));
                                    }
                                    if self.strict && rows.is_empty() {
                                        let line_no = self.idx;
                                        self.error = Some(crate::error::Error::Syntax {
                                            line: line_no,
                                            message: "empty table (no rows)".to_string(),
                                        });
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

    fn parse_scalar_line(&mut self, indent: usize) -> Value {
        let s_opt = match self.peek() {
            Some(pl) => match &pl.kind {
                LineKind::Scalar(s) => Some(*s),
                _ => None,
            },
            None => None,
        };

        // Check if this is a tabular array header
        if let Some(s) = s_opt {
            if let Some((dch, header_str)) = parse_header(s) {
                // This is a tabular header, parse it as a root-level tabular array
                if self.strict && !(dch == ',' || dch == '\t' || dch == '|') {
                    let line_no = self.idx + 1;
                    self.error = Some(crate::error::Error::Syntax {
                        line: line_no,
                        message: format!(
                            "invalid header delimiter '{}': expected ',', '\\t', or '|'",
                            dch
                        ),
                    });
                }
                self.next(); // Consume the header line

                let raw_header_tokens = split_delim_aware(header_str, dch);
                let header_keys = raw_header_tokens
                    .iter()
                    .map(|h| self.parse_key_token(h))
                    .collect::<Vec<_>>();

                if self.strict {
                    if header_keys.is_empty() {
                        let line_no = self.idx;
                        self.error = Some(crate::error::Error::Syntax {
                            line: line_no,
                            message: "empty tabular header".to_string(),
                        });
                    }
                    for htok in raw_header_tokens.iter() {
                        if !is_quoted_token(htok) && token_requires_quotes(htok, dch) {
                            let line_no = self.idx;
                            self.error = Some(crate::error::Error::Syntax {
                                line: line_no,
                                message: format!("unquoted header token requires quotes: {}", htok),
                            });
                            break;
                        }
                    }
                    for i in 0..header_keys.len() {
                        for j in (i + 1)..header_keys.len() {
                            if header_keys[i] == header_keys[j] {
                                let line_no = self.idx;
                                self.error = Some(crate::error::Error::Syntax {
                                    line: line_no,
                                    message: format!("duplicate header key: {}", header_keys[i]),
                                });
                                break;
                            }
                        }
                    }
                }

                let expected_cells = header_keys.len();
                let mut rows: Vec<Value> = Vec::new();

                loop {
                    if self.strict {
                        if let Some(bl) = self.peek() {
                            if matches!(bl.kind, LineKind::Blank) {
                                let line_no = self.idx + 1;
                                self.error = Some(crate::error::Error::Syntax {
                                    line: line_no,
                                    message: "blank line inside table".to_string(),
                                });
                            }
                        }
                    }
                    self.skip_blanks();

                    let row_item: Option<&str> = if let Some(rowl) = self.peek() {
                        if rowl.indent != indent {
                            break;
                        }
                        match &rowl.kind {
                            LineKind::ListItem { value: Some(rs) } => Some(*rs),
                            _ => None,
                        }
                    } else {
                        break;
                    };

                    let Some(rs) = row_item else {
                        break;
                    };
                    let row_line = self.idx + 1;
                    self.next();

                    let row_trimmed = rs.trim_end();
                    if self.strict && row_trimmed.as_bytes().last().copied() == Some(dch as u8) {
                        self.error = Some(crate::error::Error::Syntax {
                            line: row_line,
                            message: "trailing delimiter in row".to_string(),
                        });
                    }

                    let cells = split_delim_aware(rs, dch);
                    if self.strict && self.error.is_none() && cells.len() != expected_cells {
                        self.error = Some(crate::error::Error::Syntax {
                            line: row_line,
                            message: format!(
                                "row cell count {} does not match header {}",
                                cells.len(),
                                expected_cells
                            ),
                        });
                    }
                    if self.strict && self.error.is_none() {
                        for ctok in &cells {
                            if !is_quoted_token(ctok) && cell_token_requires_quotes(ctok, dch) {
                                self.error = Some(crate::error::Error::Syntax {
                                    line: row_line,
                                    message: format!("unquoted cell requires quotes: {}", ctok),
                                });
                                break;
                            }
                        }
                    }

                    let mut om: Vec<(String, Value)> = Vec::with_capacity(expected_cells);
                    for (i, hk) in header_keys.iter().enumerate() {
                        let cell = cells.get(i).copied().unwrap_or("null");
                        om.push((hk.clone(), self.parse_scalar_token(cell)));
                    }
                    rows.push(Value::Object(om));
                }

                if self.strict && rows.is_empty() {
                    let line_no = self.idx;
                    self.error = Some(crate::error::Error::Syntax {
                        line: line_no,
                        message: "empty table (no rows)".to_string(),
                    });
                }

                return Value::Array(rows);
            }
        }

        // Not a tabular header, parse as regular scalar
        self.next();
        if let Some(s) = s_opt {
            return self.parse_scalar_token(s);
        }
        Value::Null
    }

    fn parse_node(&mut self, indent: usize) -> Value {
        self.skip_blanks();
        let Some(line) = self.peek() else {
            return Value::Null;
        };
        // Check for empty collection syntax: [0]: for arrays, {0}: for objects
        if line.indent == indent {
            if let LineKind::KeyOnly { key } = &line.kind {
                if *key == "[0]" {
                    self.next();
                    return Value::Array(Vec::new());
                }
                if *key == "{0}" {
                    self.next();
                    return Value::Object(Vec::new());
                }
            }
        }
        match &line.kind {
            LineKind::ListItem { .. } if line.indent == indent => self.parse_array(indent),
            LineKind::KeyValue { .. } | LineKind::KeyOnly { .. } if line.indent == indent => {
                self.parse_object(indent)
            }
            LineKind::Scalar(_) if line.indent == indent => self.parse_scalar_line(indent),
            _ => {
                // If indentation doesn't match, return null
                Value::Null
            }
        }
    }

    pub fn parse_document(&mut self) -> Value {
        self.skip_blanks();
        if self.peek().is_none() {
            // Empty document represents an empty object (root documents are implicitly objects)
            return Value::Object(Vec::new());
        }
        // Check for root-level empty collection syntax: [0]: for arrays, {0}: for objects
        if let Some(line) = self.peek() {
            if line.indent == 0 {
                if let LineKind::KeyOnly { key } = &line.kind {
                    if *key == "[0]" {
                        self.next();
                        return Value::Array(Vec::new());
                    }
                    if *key == "{0}" {
                        self.next();
                        return Value::Object(Vec::new());
                    }
                }
            }
        }
        let indent = self.peek().unwrap().indent;
        self.parse_node(indent)
    }
}

fn parse_header(s: &str) -> Option<(char, &str)> {
    let mut it = s.chars();
    let at = it.next()?;
    if at != '@' {
        return None;
    }
    let dch = it.next()?;
    let rest = &s[2..];
    Some((dch, trim_ascii_start(rest)))
}

#[cfg(feature = "perf_memchr")]
fn split_delim_aware<'a>(s: &'a str, dch: char) -> Vec<&'a str> {
    let bytes = s.as_bytes();
    let delim = dch as u8;
    #[cfg(feature = "perf_smallvec")]
    let mut out: smallvec::SmallVec<[&'a str; 8]> = smallvec::SmallVec::new();
    #[cfg(not(feature = "perf_smallvec"))]
    let mut out: Vec<&'a str> = Vec::new();

    let mut in_str = false;
    let mut escape = false;
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if in_str {
            // Inside quotes, only '"' and '\\' matter; delimiter is ignored.
            if escape {
                escape = false;
                i += 1;
                continue;
            }
            if let Some(rel) = memchr::memchr2(b'"', b'\\', &bytes[i..]) {
                let idx = i + rel;
                match bytes[idx] {
                    b'\\' => {
                        escape = true;
                        i = idx + 1;
                    }
                    b'"' => {
                        in_str = false;
                        i = idx + 1;
                    }
                    _ => unreachable!(),
                }
                continue;
            } else {
                break;
            }
        } else {
            // Outside quotes, any of '"' or delimiter are interesting.
            if let Some(rel) = memchr::memchr2(b'"', delim, &bytes[i..]) {
                let idx = i + rel;
                let b = bytes[idx];
                if b == b'"' {
                    in_str = true;
                    i = idx + 1;
                    continue;
                }
                // delimiter
                let token = trim_ascii(&s[start..idx]);
                if !token.is_empty() {
                    out.push(token);
                }
                start = idx + 1;
                i = start;
                continue;
            } else {
                break;
            }
        }
    }
    if start < bytes.len() {
        let token = trim_ascii(&s[start..]);
        if !token.is_empty() {
            out.push(token);
        }
    }
    #[cfg(feature = "perf_smallvec")]
    {
        out.into_vec()
    }
    #[cfg(not(feature = "perf_smallvec"))]
    {
        out
    }
}

#[cfg(not(feature = "perf_memchr"))]
fn split_delim_aware<'a>(s: &'a str, dch: char) -> Vec<&'a str> {
    let bytes = s.as_bytes();
    #[cfg(feature = "perf_smallvec")]
    let mut out: smallvec::SmallVec<[&'a str; 8]> = smallvec::SmallVec::new();
    #[cfg(not(feature = "perf_smallvec"))]
    let mut out: Vec<&'a str> = Vec::new();
    let mut in_str = false;
    let mut escape = false;
    let mut start = 0usize;
    let delim = dch as u8;
    let len = bytes.len();
    let mut i = 0usize;
    while i < len {
        let b = bytes[i];
        if in_str {
            if escape {
                escape = false;
                i += 1;
                continue;
            }
            match b {
                b'\\' => {
                    escape = true;
                }
                b'"' => {
                    in_str = false;
                }
                _ => {}
            }
            i += 1;
            continue;
        } else {
            if b == b'"' {
                in_str = true;
                i += 1;
                continue;
            }
            if b == delim {
                let token = trim_ascii(&s[start..i]);
                if !token.is_empty() {
                    out.push(token);
                }
                start = i + 1;
            }
            i += 1;
        }
    }
    if start < len {
        let token = trim_ascii(&s[start..len]);
        if !token.is_empty() {
            out.push(token);
        }
    }
    #[cfg(feature = "perf_smallvec")]
    {
        out.into_vec()
    }
    #[cfg(not(feature = "perf_smallvec"))]
    {
        out
    }
}

fn trim_ascii(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut start = 0usize;
    let mut end = bytes.len();
    while start < end && matches!(bytes[start], b' ' | b'\t') {
        start += 1;
    }
    while end > start && matches!(bytes[end - 1], b' ' | b'\t') {
        end -= 1;
    }
    &s[start..end]
}

fn trim_ascii_start(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut start = 0usize;
    while start < bytes.len() && matches!(bytes[start], b' ' | b'\t') {
        start += 1;
    }
    &s[start..]
}

fn is_quoted_token(s: &str) -> bool {
    let t = trim_ascii(s);
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
    if s.is_empty() {
        return true;
    }
    let t = trim_ascii(s);
    if t.is_empty() {
        return true;
    }
    // Delimiter presence would have split already for unquoted tokens, but keep for safety
    if t.contains(dch) {
        return true;
    }
    // Unquoted colon is ambiguous inside cells
    if t.contains(':') {
        return true;
    }
    // Dangerous characters requiring escaping in quoted strings
    if t.chars()
        .any(|c| c == '"' || c == '\\' || is_control_char(c))
    {
        return true;
    }
    // Leading/trailing spaces would be lost; enforce quoting
    if t.starts_with(' ') || t.ends_with(' ') {
        return true;
    }
    // A lone hyphen or strings starting with hyphen (non-numeric) can be ambiguous; require quotes
    if t == "-" {
        return true;
    }
    // If it starts with '-' but is not a valid number, require quotes
    if t.starts_with('-') && t.parse::<f64>().is_err() {
        return true;
    }
    // If it starts with '+', it's numeric-like and requires quotes for clarity
    if t.starts_with('+') {
        return true;
    }
    false
}

fn is_control_char(c: char) -> bool {
    let u = c as u32;
    u < 0x20 || u == 0x7F
}

pub fn parse_to_internal_value(input: &str) -> Value {
    let mut p = Parser::from_input(input);
    p.parse_document()
}

pub fn parse_to_internal_value_from_lines<'a>(
    lines: Vec<ParsedLine<'a>>,
    strict: bool,
) -> Result<Value, crate::error::Error> {
    let mut p = Parser::from_lines(lines, strict);
    let v = p.parse_document();
    if let Some(err) = p.error {
        Err(err)
    } else {
        Ok(v)
    }
}

pub fn parse_to_value_with_strict(input: &str, strict: bool) -> Result<Value, crate::error::Error> {
    let mut p = Parser::from_input_with_strict(input, strict);
    let v = p.parse_document();
    if let Some(err) = p.error {
        Err(err)
    } else {
        Ok(v)
    }
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
            for (k, vv) in pairs {
                m.insert(k, to_json_value(vv));
            }
            serde_json::Value::Object(m)
        }
    }
}

fn unescape_json_string(s: &str) -> Option<String> {
    // Expecting a JSON string literal like "..."
    if !s.starts_with('"') || !s.ends_with('"') || s.len() < 2 {
        return None;
    }
    let inner = &s[1..s.len() - 1];
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
                    if let Some(c) = core::char::from_u32(code) {
                        out.push(c);
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        } else {
            out.push(ch);
        }
    }
    Some(out)
}

#[derive(Copy, Clone)]
enum NumHint {
    IntSigned,
    IntUnsigned,
    Float,
}

fn classify_numeric_hint(s: &str) -> Option<NumHint> {
    if s.is_empty() {
        return None;
    }
    let bytes = s.as_bytes();
    if bytes[0] == b'"' {
        return None;
    }
    let mut i = 0usize;
    if bytes[0] == b'-' || bytes[0] == b'+' {
        i = 1;
    }
    if i >= bytes.len() {
        return None;
    }
    let mut has_dot = false;
    let mut has_exp = false;
    let mut in_exponent = false;
    for &b in &bytes[i..] {
        match b {
            b'0'..=b'9' => {
                // Digits are always valid
            }
            b'.' => {
                if in_exponent {
                    // Dots not allowed in exponent
                    return None;
                }
                has_dot = true;
            }
            b'e' | b'E' => {
                if has_exp {
                    // Multiple exponent markers not allowed
                    return None;
                }
                has_exp = true;
                in_exponent = true;
            }
            b'-' | b'+' => {
                if !in_exponent {
                    // Sign only allowed in exponent (already handled leading sign)
                    return None;
                }
                // Sign is valid right after 'e'/'E', mark that we've seen it
                in_exponent = false; // Don't allow multiple signs
            }
            _ => return None,
        }
    }
    if has_dot || has_exp {
        Some(NumHint::Float)
    } else if bytes[0] == b'-' {
        Some(NumHint::IntSigned)
    } else {
        Some(NumHint::IntUnsigned)
    }
}

fn hex_val(c: char) -> Option<u32> {
    match c {
        '0'..='9' => Some((c as u32) - ('0' as u32)),
        'a'..='f' => Some(10 + (c as u32) - ('a' as u32)),
        'A'..='F' => Some(10 + (c as u32) - ('A' as u32)),
        _ => None,
    }
}
