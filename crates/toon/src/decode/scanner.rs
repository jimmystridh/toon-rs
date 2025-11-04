#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLine<'a> {
    pub indent: usize,
    pub kind: LineKind<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineKind<'a> {
    Blank,
    KeyValue { key: &'a str, value: &'a str },
    KeyOnly { key: &'a str },
    ListItem { value: Option<&'a str> },
    Scalar(&'a str),
}

#[inline]
fn leading_spaces(s: &str) -> usize {
    let b = s.as_bytes();
    let mut i = 0usize;
    while i < b.len() && b[i] == b' ' { i += 1; }
    i
}

#[inline]
#[cfg(feature = "perf_memchr")]
fn find_unquoted_colon(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut in_str = false;
    let mut escape = false;
    let mut i = 0usize;
    while i < b.len() {
        if in_str {
            if escape { escape = false; i += 1; continue; }
            if let Some(rel) = memchr::memchr2(b'"', b'\\', &b[i..]) {
                let idx = i + rel;
                match b[idx] { b'\\' => { escape = true; i = idx + 1; }, b'"' => { in_str = false; i = idx + 1; }, _ => unreachable!() }
                continue;
            } else { return None; }
        } else {
            if let Some(rel) = memchr::memchr2(b'"', b':', &b[i..]) {
                let idx = i + rel;
                match b[idx] { b'"' => { in_str = true; i = idx + 1; }, b':' => { return Some(idx); }, _ => unreachable!() }
                continue;
            } else { return None; }
        }
    }
    None
}

#[inline]
#[cfg(not(feature = "perf_memchr"))]
fn find_unquoted_colon(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut in_str = false;
    let mut escape = false;
    for i in 0..b.len() {
        let ch = b[i];
        if in_str {
            if escape { escape = false; continue; }
            match ch { b'\\' => { escape = true; }, b'"' => { in_str = false; }, _ => {} }
        } else {
            match ch { b'"' => { in_str = true; }, b':' => { return Some(i); }, _ => {} }
        }
    }
    None
}

pub fn scan<'a>(input: &'a str) -> Vec<ParsedLine<'a>> {
    let mut out = Vec::new();
    for raw in input.split_inclusive('\n') {
        let line = raw.trim_end_matches('\n');
        let indent = leading_spaces(line);
        let body = &line[indent..];
        if body.is_empty() {
            out.push(ParsedLine { indent, kind: LineKind::Blank });
            continue;
        }
        if let Some(rest) = body.strip_prefix("- ") {
            out.push(ParsedLine { indent, kind: LineKind::ListItem { value: Some(rest) } });
            continue;
        }
        if body == "-" {
            out.push(ParsedLine { indent, kind: LineKind::ListItem { value: None } });
            continue;
        }
        if body.starts_with('@') {
            // Table header line; treat entire line as scalar
            out.push(ParsedLine { indent, kind: LineKind::Scalar(body) });
            continue;
        }
        if let Some(idx) = find_unquoted_colon(body) {
            let (k, v) = body.split_at(idx);
            let after = &v[1..];
            if after.trim().is_empty() {
                out.push(ParsedLine { indent, kind: LineKind::KeyOnly { key: k } });
            } else {
                out.push(ParsedLine { indent, kind: LineKind::KeyValue { key: k, value: after.trim_start() } });
            }
            continue;
        }
        out.push(ParsedLine { indent, kind: LineKind::Scalar(body) });
    }
    out
}
