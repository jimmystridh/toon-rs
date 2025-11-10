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
    while i < b.len() && b[i] == b' ' {
        i += 1;
    }
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
            if escape {
                escape = false;
                i += 1;
                continue;
            }
            if let Some(rel) = memchr::memchr2(b'"', b'\\', &b[i..]) {
                let idx = i + rel;
                match b[idx] {
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
                return None;
            }
        } else if let Some(rel) = memchr::memchr2(b'"', b':', &b[i..]) {
            let idx = i + rel;
            match b[idx] {
                b'"' => {
                    in_str = true;
                    i = idx + 1;
                }
                b':' => {
                    return Some(idx);
                }
                _ => unreachable!(),
            }
            continue;
        } else {
            return None;
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
    for (i, &ch) in b.iter().enumerate() {
        if in_str {
            if escape {
                escape = false;
                continue;
            }
            match ch {
                b'\\' => {
                    escape = true;
                }
                b'"' => {
                    in_str = false;
                }
                _ => {}
            }
        } else {
            match ch {
                b'"' => {
                    in_str = true;
                }
                b':' => {
                    return Some(i);
                }
                _ => {}
            }
        }
    }
    None
}

pub fn scan<'a>(input: &'a str) -> Vec<ParsedLine<'a>> {
    iter(input).collect()
}

pub struct LineIter<'a> {
    rest: &'a str,
}

pub fn iter<'a>(input: &'a str) -> LineIter<'a> {
    LineIter { rest: input }
}

impl<'a> Iterator for LineIter<'a> {
    type Item = ParsedLine<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.is_empty() {
            return None;
        }
        let (raw, remaining) = match self.rest.find('\n') {
            Some(pos) => self.rest.split_at(pos + 1),
            None => {
                let line = self.rest;
                self.rest = "";
                return Some(parse_line(line));
            }
        };
        self.rest = remaining;
        Some(parse_line(raw.trim_end_matches('\n')))
    }
}

fn parse_line(line: &str) -> ParsedLine<'_> {
    let indent = leading_spaces(line);
    let body = &line[indent..];
    if body.is_empty() {
        return ParsedLine {
            indent,
            kind: LineKind::Blank,
        };
    }
    if let Some(rest) = body.strip_prefix("- ") {
        return ParsedLine {
            indent,
            kind: LineKind::ListItem { value: Some(rest) },
        };
    }
    if body == "-" {
        return ParsedLine {
            indent,
            kind: LineKind::ListItem { value: None },
        };
    }
    if body.starts_with('@') {
        return ParsedLine {
            indent,
            kind: LineKind::Scalar(body),
        };
    }
    if let Some(idx) = find_unquoted_colon(body) {
        let (k, v) = body.split_at(idx);
        let after = &v[1..];
        let after_trimmed = trim_ascii(after);
        if after_trimmed.is_empty() {
            return ParsedLine {
                indent,
                kind: LineKind::KeyOnly { key: k },
            };
        } else {
            return ParsedLine {
                indent,
                kind: LineKind::KeyValue {
                    key: k,
                    value: trim_ascii_start(after),
                },
            };
        }
    }
    ParsedLine {
        indent,
        kind: LineKind::Scalar(body),
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
