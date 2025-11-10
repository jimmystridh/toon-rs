use js_sys::{Array, Object, Reflect};
use toon::decode::scanner::{self, LineIter, LineKind, ParsedLine};
use wasm_bindgen::JsValue;

use std::{string::String, vec::Vec};

pub fn parse_input_to_js(input: &str, strict: bool) -> Result<JsValue, JsValue> {
    let iter = scanner::iter(input);
    JsParser::from_iter(iter).parse(strict)
}

pub fn parse_lines_to_js<'a>(lines: Vec<ParsedLine<'a>>, strict: bool) -> Result<JsValue, JsValue> {
    JsParser::from_vec(lines).parse(strict)
}

struct LineStream<'a> {
    iter: Option<LineIter<'a>>,
    vec: Option<Vec<ParsedLine<'a>>>,
    idx: usize,
    peeked: Option<ParsedLine<'a>>,
}

impl<'a> LineStream<'a> {
    fn from_iter(iter: LineIter<'a>) -> Self {
        Self {
            iter: Some(iter),
            vec: None,
            idx: 0,
            peeked: None,
        }
    }

    fn from_vec(vec: Vec<ParsedLine<'a>>) -> Self {
        Self {
            iter: None,
            vec: Some(vec),
            idx: 0,
            peeked: None,
        }
    }

    fn next(&mut self) -> Option<ParsedLine<'a>> {
        if let Some(line) = self.peeked.take() {
            return Some(line);
        }
        if let Some(iter) = &mut self.iter {
            iter.next()
        } else if let Some(vec) = &self.vec {
            if self.idx < vec.len() {
                let line = vec[self.idx].clone();
                self.idx += 1;
                Some(line)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn peek(&mut self) -> Option<&ParsedLine<'a>> {
        if self.peeked.is_none() {
            self.peeked = self.next();
        }
        self.peeked.as_ref()
    }
}

pub struct JsParser<'a> {
    lines: LineStream<'a>,
    error: Option<JsValue>,
}

impl<'a> JsParser<'a> {
    fn from_iter(iter: LineIter<'a>) -> Self {
        Self {
            lines: LineStream::from_iter(iter),
            error: None,
        }
    }

    fn from_vec(vec: Vec<ParsedLine<'a>>) -> Self {
        Self {
            lines: LineStream::from_vec(vec),
            error: None,
        }
    }

    fn parse(mut self, _strict: bool) -> Result<JsValue, JsValue> {
        let value = self.parse_document();
        if let Some(err) = self.error {
            Err(err)
        } else {
            Ok(value)
        }
    }

    fn skip_blanks(&mut self) {
        while let Some(line) = self.lines.peek() {
            if matches!(line.kind, LineKind::Blank) {
                self.lines.next();
            } else {
                break;
            }
        }
    }

    fn peek(&mut self) -> Option<ParsedLine<'a>> {
        self.lines.peek().cloned()
    }

    fn next(&mut self) -> Option<ParsedLine<'a>> {
        self.lines.next()
    }

    fn parse_scalar_token(&self, token: &str) -> JsValue {
        if token.starts_with('"') {
            if let Some(unquoted) = unescape_json_string(token) {
                return JsValue::from_str(&unquoted);
            }
        }
        if has_forbidden_leading_zeros(token) {
            return JsValue::from_str(token);
        }
        match token {
            "null" => return JsValue::NULL,
            "true" => return JsValue::from_bool(true),
            "false" => return JsValue::from_bool(false),
            _ => {}
        }
        if let Some(hint) = classify_numeric_hint(token) {
            match hint {
                NumHint::IntSigned | NumHint::IntUnsigned | NumHint::Float => {
                    if let Ok(num) = token.parse::<f64>() {
                        return JsValue::from_f64(num);
                    }
                }
            }
        }
        JsValue::from_str(token)
    }

    fn parse_key_token(&self, key: &str) -> String {
        if key.starts_with('"') {
            if let Some(unquoted) = unescape_json_string(key) {
                return unquoted;
            }
        }
        key.to_string()
    }

    fn parse_array(&mut self, indent: usize) -> JsValue {
        let array = Array::new();
        loop {
            self.skip_blanks();
            let Some(line) = self.peek() else { break };
            if line.indent != indent {
                break;
            }
            match line.kind {
                LineKind::ListItem { value } => {
                    self.next();
                    if let Some(token) = value {
                        array.push(&self.parse_scalar_token(token));
                    } else {
                        let child = indent + 2;
                        array.push(&self.parse_node(child));
                    }
                }
                _ => break,
            }
        }
        array.into()
    }

    fn parse_object(&mut self, indent: usize) -> JsValue {
        let obj = Object::new();
        loop {
            self.skip_blanks();
            let Some(line) = self.peek() else { break };
            if line.indent != indent {
                break;
            }
            match line.kind {
                LineKind::KeyValue { key, value } => {
                    self.next();
                    let k = self.parse_key_token(key);
                    let v = self.parse_scalar_token(value);
                    set_prop(&obj, &k, &v);
                }
                LineKind::KeyOnly { key } => {
                    self.next();
                    let k = self.parse_key_token(key);
                    let child = indent + 2;
                    let v = self.parse_node(child);
                    set_prop(&obj, &k, &v);
                }
                _ => break,
            }
        }
        obj.into()
    }

    fn parse_scalar_line(&mut self, indent: usize) -> JsValue {
        let Some(line) = self.peek() else {
            return JsValue::NULL;
        };
        if line.indent != indent {
            return JsValue::NULL;
        }
        if let LineKind::Scalar(text) = line.kind {
            if let Some((delim, rest)) = parse_header(text) {
                self.next();
                if let Some(value) = self.parse_tabular(indent, delim, rest) {
                    return value;
                }
                return JsValue::NULL;
            }
            self.next();
            return self.parse_scalar_token(text);
        }
        JsValue::NULL
    }

    fn parse_tabular(&mut self, indent: usize, delim: char, rest: &str) -> Option<JsValue> {
        let mut header_slices = Vec::new();
        if !split_cells(rest, delim, &mut header_slices) {
            self.error = Some(JsValue::from_str("invalid tabular header"));
            return None;
        }
        if header_slices.is_empty() {
            self.error = Some(JsValue::from_str("empty tabular header"));
            return None;
        }
        let headers: Vec<String> = header_slices
            .iter()
            .map(|h| self.parse_key_token(h))
            .collect();
        let mut cell_slices: Vec<&str> = Vec::with_capacity(headers.len());
        let rows = Array::new();
        loop {
            self.skip_blanks();
            let Some(line) = self.peek() else { break };
            if line.indent != indent {
                break;
            }
            match line.kind {
                LineKind::ListItem { value: Some(row) } => {
                    self.next();
                    if !split_cells(row, delim, &mut cell_slices) {
                        self.error = Some(JsValue::from_str("invalid tabular row"));
                        return None;
                    }
                    if cell_slices.len() != headers.len() {
                        self.error = Some(JsValue::from_str("tabular row width mismatch"));
                        return None;
                    }
                    let obj = Object::new();
                    for (header, cell) in headers.iter().zip(cell_slices.iter()) {
                        let key = self.parse_key_token(header);
                        let value = self.parse_scalar_token(cell);
                        set_prop(&obj, &key, &value);
                    }
                    rows.push(&obj.into());
                }
                _ => break,
            }
        }
        Some(rows.into())
    }

    fn parse_node(&mut self, indent: usize) -> JsValue {
        self.skip_blanks();
        let Some(line) = self.peek() else {
            return JsValue::NULL;
        };
        if line.indent == indent {
            if let LineKind::KeyOnly { key } = line.kind {
                if key == "[0]" {
                    self.next();
                    return Array::new().into();
                }
                if key == "{0}" {
                    self.next();
                    return Object::new().into();
                }
            }
        }
        match line.kind {
            LineKind::ListItem { .. } if line.indent == indent => self.parse_array(indent),
            LineKind::KeyValue { .. } | LineKind::KeyOnly { .. } if line.indent == indent => {
                self.parse_object(indent)
            }
            LineKind::Scalar(_) if line.indent == indent => self.parse_scalar_line(indent),
            _ => JsValue::NULL,
        }
    }

    fn parse_document(&mut self) -> JsValue {
        self.skip_blanks();
        if self.peek().is_none() {
            return Object::new().into();
        }
        if let Some(line) = self.peek() {
            if line.indent == 0 {
                if let LineKind::KeyOnly { key } = line.kind {
                    if key == "[0]" {
                        self.next();
                        return Array::new().into();
                    }
                    if key == "{0}" {
                        self.next();
                        return Object::new().into();
                    }
                }
            }
        }
        let indent = self.peek().map(|l| l.indent).unwrap_or(0);
        self.parse_node(indent)
    }
}

fn set_prop(obj: &Object, key: &str, value: &JsValue) {
    let _ = Reflect::set(obj, &JsValue::from_str(key), value);
}

#[derive(Clone, Copy)]
enum NumHint {
    IntSigned,
    IntUnsigned,
    Float,
}

fn classify_numeric_hint(token: &str) -> Option<NumHint> {
    let t = token.trim();
    if t.is_empty() {
        return None;
    }
    if t.starts_with('-') {
        if t[1..].chars().all(|c| c.is_ascii_digit()) {
            return Some(NumHint::IntSigned);
        }
    } else if t.chars().all(|c| c.is_ascii_digit()) {
        return Some(NumHint::IntUnsigned);
    }
    if t.contains('.') || t.contains('e') || t.contains('E') {
        if t.parse::<f64>().is_ok() {
            return Some(NumHint::Float);
        }
    }
    None
}

fn has_forbidden_leading_zeros(token: &str) -> bool {
    let token = token.trim();
    if token.is_empty() {
        return false;
    }
    let token = token.strip_prefix('-').unwrap_or(token);
    let token = token.strip_prefix('+').unwrap_or(token);
    if token.len() <= 1 {
        return false;
    }
    let first = token.as_bytes()[0];
    if first != b'0' {
        return false;
    }
    let second = token.as_bytes()[1];
    if second == b'.' || second == b'e' || second == b'E' {
        return false;
    }
    true
}

fn unescape_json_string(token: &str) -> Option<String> {
    if !token.starts_with('"') || !token.ends_with('"') {
        return None;
    }
    let mut out = String::with_capacity(token.len() - 2);
    let mut chars = token[1..token.len() - 1].chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next) = chars.next() {
                    match next {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        '/' => out.push('/'),
                        'b' => out.push('\u{0008}'),
                        'f' => out.push('\u{000C}'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => {
                            let mut hex = String::new();
                            for _ in 0..4 {
                                if let Some(h) = chars.next() {
                                    hex.push(h);
                                } else {
                                    return None;
                                }
                            }
                            if let Ok(code) = u16::from_str_radix(&hex, 16) {
                                if let Some(ch) = core::char::from_u32(code as u32) {
                                    out.push(ch);
                                }
                            }
                        }
                        _ => out.push(next),
                    }
                }
            }
            _ => out.push(c),
        }
    }
    Some(out)
}

fn parse_header(s: &str) -> Option<(char, &str)> {
    let mut chars = s.chars();
    if chars.next()? != '@' {
        return None;
    }
    let dch = chars.next()?;
    let rest = &s[2..];
    Some((dch, rest.trim_start()))
}

fn split_cells<'a>(line: &'a str, dch: char, out: &mut Vec<&'a str>) -> bool {
    out.clear();
    let bytes = line.as_bytes();
    let mut start = 0usize;
    let mut i = 0usize;
    let mut in_str = false;
    let mut escape = false;
    while i < bytes.len() {
        let b = bytes[i];
        if escape {
            escape = false;
            i += 1;
            continue;
        }
        if b == b'\\' && in_str {
            escape = true;
            i += 1;
            continue;
        }
        if b == b'"' {
            in_str = !in_str;
            i += 1;
            continue;
        }
        if !in_str && (b as char) == dch {
            out.push(trim_segment(line, start, i));
            i += 1;
            if dch != '\t' && i < bytes.len() && bytes[i] == b' ' {
                i += 1;
            }
            start = i;
            continue;
        }
        i += 1;
    }
    if in_str {
        return false;
    }
    out.push(trim_segment(line, start, bytes.len()));
    true
}

fn trim_segment<'a>(line: &'a str, mut start: usize, mut end: usize) -> &'a str {
    let bytes = line.as_bytes();
    while start < end && matches!(bytes[start], b' ' | b'\t') {
        start += 1;
    }
    while end > start && matches!(bytes[end - 1], b' ' | b'\t') {
        end -= 1;
    }
    &line[start..end]
}
