use serde::de::{self, DeserializeOwned, IntoDeserializer, MapAccess, SeqAccess};

use crate::{error::Error as ToONError, options::Options, Result};
use crate::decode::scanner::{scan, ParsedLine, LineKind};

#[derive(Debug)]
pub struct DeError { msg: String }
impl core::fmt::Display for DeError { fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { f.write_str(&self.msg) } }
impl de::Error for DeError { fn custom<T: core::fmt::Display>(t: T) -> Self { DeError { msg: t.to_string() } } }
impl core::error::Error for DeError {}

pub fn from_str<T: DeserializeOwned>(s: &str, options: &Options) -> Result<T> {
    let lines = scan(s);
    if options.strict {
        if let Err(e) = crate::decode::validation::validate_indentation(&lines) {
            return Err(ToONError::Syntax { line: e.line, message: e.message });
        }
    }
    let mut de = DirectDeserializer { lines, idx: 0, strict: options.strict };
    T::deserialize(&mut de).map_err(|e: DeError| ToONError::Message(e.msg))
}

struct DirectDeserializer<'a> {
    lines: Vec<ParsedLine<'a>>,
    idx: usize,
    strict: bool,
}

impl<'a> DirectDeserializer<'a> {
    fn skip_blanks(&mut self) {
        while let Some(pl) = self.lines.get(self.idx) {
            if matches!(pl.kind, LineKind::Blank) { self.idx += 1; } else { break; }
        }
    }
    fn peek(&self) -> Option<&ParsedLine<'a>> { self.lines.get(self.idx) }
    fn next(&mut self) -> Option<&ParsedLine<'a>> { let i = self.idx; self.idx += 1; self.lines.get(i) }

    fn parse_key(&self, k: &str) -> String {
        if let Some(s) = unescape_json_string(k) { return s; }
        let base = if let Some(idx) = k.find('[') { &k[..idx] } else { k };
        base.to_string()
    }

    fn classify_primitive(s: &str) -> Primitive {
        if let Some(st) = unescape_json_string(s) { return Primitive::Str(st); }
        match s {
            "true" => return Primitive::Bool(true),
            "false" => return Primitive::Bool(false),
            "null" => return Primitive::Null,
            _ => {}
        }
        let bs = s.as_bytes();
        if !bs.is_empty() {
            if bs[0] == b'-' { if bs.len()>1 && bs[1..].iter().all(|c| c.is_ascii_digit()) { if let Ok(i)=s.parse::<i64>() { return Primitive::I64(i);} } }
            else if bs.iter().all(|c| c.is_ascii_digit()) { if let Ok(u)=s.parse::<u64>() { return Primitive::U64(u);} }
        }
        if let Ok(f)=s.parse::<f64>() { return Primitive::F64(f); }
        Primitive::Str(s.to_string())
    }
}

impl<'de, 'a> de::Deserializer<'de> for &mut DirectDeserializer<'a> {
    type Error = DeError;

    fn deserialize_any<V>(mut self, visitor: V) -> core::result::Result<V::Value, Self::Error>
    where V: de::Visitor<'de> {
        self.skip_blanks();
        let Some(pl) = self.peek() else { return visitor.visit_unit(); };
        let indent = pl.indent;
        match &pl.kind {
            LineKind::ListItem { .. } => {
                // Root is an array
                let mut sa = SeqDe { de: self, indent };
                visitor.visit_seq(&mut sa)
            }
            LineKind::KeyValue { .. } | LineKind::KeyOnly { .. } => {
                let mut ma = MapDe { de: self, indent, pending: None };
                visitor.visit_map(&mut ma)
            }
            LineKind::Scalar(s) => {
                let tok = DirectDeserializer::classify_primitive(s);
                PrimDe(tok).deserialize_any(visitor)
            }
            LineKind::Blank => visitor.visit_unit(),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map struct
        enum identifier ignored_any
    }
}

enum Primitive { Null, Bool(bool), I64(i64), U64(u64), F64(f64), Str(String) }
struct PrimDe(Primitive);
impl<'de> de::Deserializer<'de> for PrimDe {
    type Error = DeError;
    fn deserialize_any<V>(self, visitor: V) -> core::result::Result<V::Value, Self::Error>
    where V: de::Visitor<'de> {
        match self.0 {
            Primitive::Null => visitor.visit_unit(),
            Primitive::Bool(b) => visitor.visit_bool(b),
            Primitive::I64(i) => visitor.visit_i64(i),
            Primitive::U64(u) => visitor.visit_u64(u),
            Primitive::F64(f) => visitor.visit_f64(f),
            Primitive::Str(s) => visitor.visit_string(s),
        }
    }
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct map struct
        enum identifier ignored_any
    }
}

struct SeqDe<'a, 'b> { de: &'b mut DirectDeserializer<'a>, indent: usize }
impl<'de, 'a, 'b> SeqAccess<'de> for SeqDe<'a, 'b> {
    type Error = DeError;
    fn next_element_seed<T>(&mut self, seed: T) -> core::result::Result<Option<T::Value>, Self::Error>
    where T: de::DeserializeSeed<'de> {
        self.de.skip_blanks();
        // Snapshot to avoid borrowing across next()
        let val_opt: Option<&str> = if let Some(pl) = self.de.peek() {
            if pl.indent != self.indent { return Ok(None); }
            match &pl.kind { LineKind::ListItem { value } => value.clone(), _ => return Ok(None) }
        } else { return Ok(None) };
        self.de.next();
        if let Some(vs) = val_opt {
            // 1) Inline primitive array item: "[N<delim?>]: v1<delim>..."
            if vs.starts_with('[') {
                if let Some((_n, dch, values_str)) = parse_inline_array_header(vs) {
                    let toks = split_delim_aware(values_str, dch);
                    let mut ia = InlineArraySeq { tokens: toks, idx: 0 };
                    return seed.deserialize(de::value::SeqAccessDeserializer::new(&mut ia)).map(Some);
                }
            }
            // 2) Object on hyphen line: variants
            if let Some(colon) = find_unquoted_colon(vs) {
                let (kraw, rest) = vs.split_at(colon);
                let after = rest[1..].trim_start();
                // 2a) Tabular header on hyphen line: key[N]{fields}:
                if after.is_empty() {
                    if let Some((key, dch, header)) = try_parse_keyed_tabular_header(self.de, kraw) {
                        let mut obj = HyphenObjectDe { de: self.de, first_key: Some(key), first_val: Some(HyphenFirstValue::TabularHeader { dch, header }), siblings_indent: self.indent + 1 };
                        return seed.deserialize(de::value::MapAccessDeserializer::new(&mut obj)).map(Some);
                    }
                }
                // 2b) Regular key: value or nested object
                let key = self.de.parse_key(kraw);
                // First field value can be scalar, inline array header, or nested object
                let first_val = if after.is_empty() {
                    HyphenFirstValue::NestedObject { child_indent: self.indent + 2 }
                } else if after.starts_with('[') {
                    if let Some((_n, dch, values_str)) = parse_inline_array_header(after) {
                        HyphenFirstValue::InlineArray { dch, values_str }
                    } else { HyphenFirstValue::Scalar(after) }
                } else { HyphenFirstValue::Scalar(after) };
                let mut obj = HyphenObjectDe { de: self.de, first_key: Some(key), first_val: Some(first_val), siblings_indent: self.indent + 2 };
                return seed.deserialize(de::value::MapAccessDeserializer::new(&mut obj)).map(Some);
            }
            // 3) Fallback primitive value
            let tok = DirectDeserializer::classify_primitive(vs);
            let de = PrimDe(tok);
            seed.deserialize(de).map(Some)
        } else {
            // "-" as empty object
            let mut ma = MapDe { de: self.de, indent: self.indent + 2, pending: None };
            seed.deserialize(de::value::MapAccessDeserializer::new(&mut ma)).map(Some)
        }
    }
}

struct MapDe<'a, 'b> {
    de: &'b mut DirectDeserializer<'a>,
    indent: usize,
    pending: Option<(String, ValueKind<'a>)>,
}

enum ValueKind<'a> {
    Scalar(&'a str),
    InlinePrimitiveArray { dch: char, values_str: &'a str },
    NestedObject { child_indent: usize },
    Array { child_indent: usize },
    Tabular { dch: char, header: Vec<String>, child_indent: usize },
}

impl<'de, 'a, 'b> MapAccess<'de> for MapDe<'a, 'b> {
    type Error = DeError;

    fn next_key_seed<K>(&mut self, seed: K) -> core::result::Result<Option<K::Value>, Self::Error>
    where K: de::DeserializeSeed<'de> {
        self.de.skip_blanks();
        // Snapshot key/value before advancing
        let snapshot = if let Some(pl) = self.de.peek() {
            if pl.indent != self.indent { return Ok(None); }
            match &pl.kind {
                LineKind::KeyValue { key, value } => (Some(*key), Some(*value)),
                LineKind::KeyOnly { key } => (Some(*key), None),
                _ => return Ok(None),
            }
        } else { return Ok(None) };
        match snapshot {
            (Some(kref), Some(vref)) => {
                self.de.next();
                let k = self.de.parse_key(kref);
                // Inline primitive arrays as object field:
                // Either value starts with header, or key contains [N<delim?>]
                if vref.starts_with('[') {
                    if let Some((_n, dch, values_str)) = parse_inline_array_header(vref) {
                        self.pending = Some((k.clone(), ValueKind::InlinePrimitiveArray { dch, values_str }));
                        return seed.deserialize(k.into_deserializer()).map(Some);
                    }
                } else if kref.contains('[') {
                    let dch = bracket_delim_from_key(kref).unwrap_or(',');
                    self.pending = Some((k.clone(), ValueKind::InlinePrimitiveArray { dch, values_str: vref }));
                    return seed.deserialize(k.into_deserializer()).map(Some);
                }
                self.pending = Some((k.clone(), ValueKind::Scalar(vref)));
                seed.deserialize(k.into_deserializer()).map(Some)
            }
            (Some(kref), None) => {
                self.de.next();
                let k = self.de.parse_key(kref);
                let child_indent = self.indent + 2;
                // Distinguish keyed header vs nested object
                let vkind = if kref.contains('[') {
                    // Try keyed tabular header "key[N]{fields}:"
                    if let Some((_key_again, dch, header)) = try_parse_keyed_tabular_header(self.de, kref) {
                        ValueKind::Tabular { dch, header, child_indent }
                    } else {
                        // key[N]: list array items under child indent
                        ValueKind::Array { child_indent }
                    }
                } else {
                    // Not header: regular nested object
                    // Also support legacy next-line scalar header starting with '@'
                    if let Some(nl) = self.de.peek() {
                        if nl.indent == child_indent {
                            if let LineKind::Scalar(s) = &nl.kind {
                                if let Some((dch, hdr)) = parse_header(s) {
                                    self.de.next();
                                    let fields = split_delim_aware(hdr, dch).into_iter().map(|f| self.de.parse_key(f)).collect::<Vec<_>>();
                                    ValueKind::Tabular { dch, header: fields, child_indent }
                                } else { ValueKind::NestedObject { child_indent } }
                            } else { ValueKind::NestedObject { child_indent } }
                        } else { ValueKind::NestedObject { child_indent } }
                    } else { ValueKind::NestedObject { child_indent } }
                };
                self.pending = Some((k.clone(), vkind));
                seed.deserialize(k.into_deserializer()).map(Some)
            }
            _ => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> core::result::Result<V::Value, Self::Error>
    where V: de::DeserializeSeed<'de> {
        let (_k, vk) = self.pending.take().ok_or_else(|| DeError{ msg: "value requested without key".into() })?;
        match vk {
            ValueKind::Scalar(s) => seed.deserialize(PrimDe(DirectDeserializer::classify_primitive(s))),
            ValueKind::InlinePrimitiveArray { dch, values_str } => {
                let toks = split_delim_aware(values_str, dch);
                let mut ia = InlineArraySeq { tokens: toks, idx: 0 };
                seed.deserialize(de::value::SeqAccessDeserializer::new(&mut ia))
            }
            ValueKind::NestedObject { child_indent } => {
                let mut ma = MapDe { de: self.de, indent: child_indent, pending: None };
                seed.deserialize(de::value::MapAccessDeserializer::new(&mut ma))
            }
            ValueKind::Array { child_indent } => {
                let mut sa = SeqDe { de: self.de, indent: child_indent };
                seed.deserialize(de::value::SeqAccessDeserializer::new(&mut sa))
            }
            ValueKind::Tabular { dch, header, child_indent } => {
                let mut ta = TabularSeqDe { de: self.de, indent: child_indent, dch, header };
                seed.deserialize(de::value::SeqAccessDeserializer::new(&mut ta))
            }
        }
    }
}

struct TabularSeqDe<'a, 'b> { de: &'b mut DirectDeserializer<'a>, indent: usize, dch: char, header: Vec<String> }

struct InlineArraySeq<'a> { tokens: Vec<&'a str>, idx: usize }
impl<'de, 'a> SeqAccess<'de> for InlineArraySeq<'a> {
    type Error = DeError;
    fn next_element_seed<T>(&mut self, seed: T) -> core::result::Result<Option<T::Value>, Self::Error>
    where T: de::DeserializeSeed<'de> {
        if self.idx >= self.tokens.len() { return Ok(None); }
        let s = self.tokens[self.idx];
        self.idx += 1;
        let de = PrimDe(DirectDeserializer::classify_primitive(s));
        seed.deserialize(de).map(Some)
    }
}

struct HyphenObjectDe<'a, 'b> { de: &'b mut DirectDeserializer<'a>, first_key: Option<String>, first_val: Option<HyphenFirstValue<'a>>, siblings_indent: usize }

enum HyphenFirstValue<'a> {
    Scalar(&'a str),
    InlineArray { dch: char, values_str: &'a str },
    NestedObject { child_indent: usize },
    TabularHeader { dch: char, header: Vec<String> },
}

impl<'de, 'a, 'b> MapAccess<'de> for HyphenObjectDe<'a, 'b> {
    type Error = DeError;
    fn next_key_seed<K>(&mut self, seed: K) -> core::result::Result<Option<K::Value>, Self::Error>
    where K: de::DeserializeSeed<'de> {
        if let Some(k) = self.first_key.take() { return seed.deserialize(k.into_deserializer()).map(Some); }
        // After the first field, parse siblings at siblings_indent
        self.de.skip_blanks();
        let snapshot = if let Some(pl) = self.de.peek() {
            if pl.indent != self.siblings_indent { return Ok(None); }
            match &pl.kind {
                LineKind::KeyValue { key, value } => (Some(*key), Some(*value)),
                LineKind::KeyOnly { key } => (Some(*key), None),
                _ => return Ok(None),
            }
        } else { return Ok(None) };
        self.de.next();
        let k = self.de.parse_key(snapshot.0.unwrap());
        // Stash the value kind back into first_val slot to reuse next_value_seed path
        if let Some(v) = snapshot.1 { self.first_val = Some(HyphenFirstValue::Scalar(v)); } else { self.first_val = Some(HyphenFirstValue::Scalar("null")); }
        seed.deserialize(k.into_deserializer()).map(Some)
    }
    fn next_value_seed<V>(&mut self, seed: V) -> core::result::Result<V::Value, Self::Error>
    where V: de::DeserializeSeed<'de> {
        if let Some(fv) = self.first_val.take() {
            match fv {
                HyphenFirstValue::Scalar(s) => seed.deserialize(PrimDe(DirectDeserializer::classify_primitive(s))),
                HyphenFirstValue::InlineArray { dch, values_str } => {
                    let toks = split_delim_aware(values_str, dch);
                    let mut ia = InlineArraySeq { tokens: toks, idx: 0 };
                    seed.deserialize(de::value::SeqAccessDeserializer::new(&mut ia))
                }
                HyphenFirstValue::NestedObject { child_indent } => {
                    let mut ma = MapDe { de: self.de, indent: child_indent, pending: None };
                    seed.deserialize(de::value::MapAccessDeserializer::new(&mut ma))
                }
                HyphenFirstValue::TabularHeader { dch, header } => {
                    let mut ta = TabularSeqDe { de: self.de, indent: self.siblings_indent, dch, header };
                    seed.deserialize(de::value::SeqAccessDeserializer::new(&mut ta))
                }
            }
        } else {
            // Should not happen if next_key_seed is correct
            Err(DeError{ msg: "value requested without pending field".into() })
        }
    }
}
impl<'de, 'a, 'b> SeqAccess<'de> for TabularSeqDe<'a, 'b> {
    type Error = DeError;
    fn next_element_seed<T>(&mut self, seed: T) -> core::result::Result<Option<T::Value>, Self::Error>
    where T: de::DeserializeSeed<'de> {
        self.de.skip_blanks();
        // Snapshot row string before advancing
        let rs_opt: Option<&str> = if let Some(pl) = self.de.peek() {
            if pl.indent != self.indent { return Ok(None); }
            match &pl.kind { LineKind::ListItem { value: Some(rs) } => Some(*rs), _ => return Ok(None) }
        } else { return Ok(None) };
        let row_line = self.de.idx + 1;
        self.de.next();
        let rs = rs_opt.unwrap();
        let cells = split_delim_aware(rs, self.dch);
            if self.de.strict && cells.len() != self.header.len() {
                return Err(DeError{ msg: format!("row cell count {} does not match header {} at line {}", cells.len(), self.header.len(), row_line) });
            }
            let mut rma = RowMapDe { header: &self.header, cells, idx: 0 };
            seed.deserialize(de::value::MapAccessDeserializer::new(&mut rma)).map(Some)
    }
}

struct RowMapDe<'h> { header: &'h [String], cells: Vec<&'h str>, idx: usize }
impl<'de, 'h> MapAccess<'de> for RowMapDe<'h> {
    type Error = DeError;
    fn next_key_seed<K>(&mut self, seed: K) -> core::result::Result<Option<K::Value>, Self::Error>
    where K: de::DeserializeSeed<'de> {
        if self.idx >= self.header.len() { return Ok(None); }
        let key = &self.header[self.idx];
        seed.deserialize(key.clone().into_deserializer()).map(Some)
    }
    fn next_value_seed<V>(&mut self, seed: V) -> core::result::Result<V::Value, Self::Error>
    where V: de::DeserializeSeed<'de> {
        let i = self.idx; self.idx += 1;
        let s = self.cells.get(i).copied().unwrap_or("null");
        seed.deserialize(PrimDe(DirectDeserializer::classify_primitive(s)))
    }
}

fn parse_header(s: &str) -> Option<(char, &str)> {
    let mut it = s.chars();
    if it.next()? != '@' { return None; }
    let dch = it.next()?;
    Some((dch, s[2..].trim_start()))
}

fn unescape_json_string(s: &str) -> Option<String> {
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
                    for _ in 0..4 { let d = chars.next()?; code = (code << 4) | hex_val(d)?; }
                    if let Some(c) = core::char::from_u32(code) { out.push(c); } else { return None; }
                }
                _ => return None,
            }
        } else { out.push(ch); }
    }
    Some(out)
}

fn hex_val(c: char) -> Option<u32> {
    match c { '0'..='9' => Some((c as u32) - ('0' as u32)), 'a'..='f' => Some(10 + (c as u32) - ('a' as u32)), 'A'..='F' => Some(10 + (c as u32) - ('A' as u32)), _ => None }
}

fn split_delim_aware<'a>(s: &'a str, dch: char) -> Vec<&'a str> {
    #[cfg(feature = "perf_memchr")]
    {
        // Reuse parser's optimized splitter if available via cfg; else local slow path
    }
    let bytes = s.as_bytes();
    let mut out: Vec<&'a str> = Vec::new();
    let mut in_str = false; let mut escape = false; let mut start = 0usize; let delim = dch as u8;
    for (i, &b) in bytes.iter().enumerate() {
        if in_str {
            if escape { escape = false; continue; }
            match b { b'\\' => { escape = true; }, b'"' => { in_str = false; }, _ => {} }
            continue;
        } else {
            if b == b'"' { in_str = true; continue; }
            if b == delim { let tok = s[start..i].trim(); if !tok.is_empty() { out.push(tok); } start = i + 1; }
        }
    }
    if start < bytes.len() { let tok = s[start..].trim(); if !tok.is_empty() { out.push(tok); } }
    out
}

fn find_unquoted_colon(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut in_str = false; let mut escape = false;
    for (i,&ch) in b.iter().enumerate() {
        if in_str { if escape { escape=false; continue; } match ch { b'\\' => { escape=true; }, b'"' => { in_str=false; }, _=>{} } }
        else { match ch { b'"' => { in_str=true; }, b':' => { return Some(i); }, _=>{} } }
    }
    None
}

fn parse_inline_array_header(s: &str) -> Option<(usize, char, &str)> {
    // Expect: "[N<delim?>]:" followed by values
    let bytes = s.as_bytes();
    if bytes.first().copied()? != b'[' { return None; }
    let mut i = 1usize; let mut n: usize = 0; while i < bytes.len() && bytes[i].is_ascii_digit() { n = n*10 + (bytes[i]-b'0') as usize; i+=1; }
    if i >= bytes.len() { return None; }
    let mut dch = ',';
    match bytes[i] { b'\t' => { dch='\t'; i+=1; }, b'|' => { dch='|'; i+=1; }, _ => {} }
    if i >= bytes.len() || bytes[i] != b']' { return None; }
    i+=1;
    if i >= bytes.len() || bytes[i] != b':' { return None; }
    let rest = s[i+1..].trim_start();
    Some((n, dch, rest))
}

fn try_parse_keyed_tabular_header<'a>(de: &DirectDeserializer<'a>, kraw: &str) -> Option<(String, char, Vec<String>)> {
    let bytes = kraw.as_bytes();
    let br = bytes.iter().position(|&b| b == b'[')?; // require '['
    let key_str = &kraw[..br];
    let mut i = br + 1; let mut dch = ',';
    // Skip length digits
    while i < bytes.len() && bytes[i].is_ascii_digit() { i += 1; }
    if i < bytes.len() {
        match bytes[i] { b'\t' => { dch='\t'; i+=1; }, b'|' => { dch='|'; i+=1; }, _ => {} }
    }
    if i >= bytes.len() || bytes[i] != b']' { return None; }
    i += 1;
    // Require fields segment for tabular; otherwise not a tabular header
    if !(i < bytes.len() && bytes[i] == b'{') { return None; }
    // find matching '}'
    let close = kraw[i..].find('}')? + i;
    let inner = &kraw[i+1..close];
    let mut header: Vec<String> = Vec::new();
    for tok in split_delim_aware(inner, dch) { header.push(de.parse_key(tok)); }
    i = close + 1;
    // trailing content must be empty
    if i != bytes.len() { return None; }
    let key = de.parse_key(key_str);
    Some((key, dch, header))
}

fn bracket_delim_from_key(k: &str) -> Option<char> {
    let bytes = k.as_bytes();
    let bpos = bytes.iter().position(|&b| b == b'[')?;
    let mut i = bpos + 1;
    while i < bytes.len() && bytes[i].is_ascii_digit() { i += 1; }
    if i < bytes.len() {
        match bytes[i] { b'\t' => return Some('\t'), b'|' => return Some('|'), _ => {} }
    }
    Some(',')
}
