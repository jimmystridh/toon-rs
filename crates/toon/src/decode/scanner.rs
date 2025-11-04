#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLine {
    pub indent: usize,
    pub kind: LineKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineKind {
    Blank,
    KeyValue { key: String, value: String },
    KeyOnly { key: String },
    ListItem { value: Option<String> },
    Scalar(String),
}

fn leading_spaces(s: &str) -> usize {
    s.chars().take_while(|c| *c == ' ').count()
}

fn find_unquoted_colon(s: &str) -> Option<usize> {
    let mut in_str = false;
    let mut escape = false;
    for (i, ch) in s.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        match ch {
            '\\' if in_str => {
                escape = true;
            }
            '"' => {
                in_str = !in_str;
            }
            ':' if !in_str => {
                return Some(i);
            }
            _ => {}
        }
    }
    None
}

pub fn scan(input: &str) -> Vec<ParsedLine> {
    let mut out = Vec::new();
    for raw in input.split_inclusive('\n') {
        let line = raw.trim_end_matches('\n');
        let indent = leading_spaces(line);
        let body = &line[indent..];
        if body.is_empty() {
            out.push(ParsedLine {
                indent,
                kind: LineKind::Blank,
            });
            continue;
        }
        if let Some(rest) = body.strip_prefix("- ") {
            out.push(ParsedLine {
                indent,
                kind: LineKind::ListItem {
                    value: Some(rest.to_string()),
                },
            });
            continue;
        }
        if body == "-" {
            out.push(ParsedLine {
                indent,
                kind: LineKind::ListItem { value: None },
            });
            continue;
        }
        if body.starts_with('@') {
            // Table header line; treat entire line as scalar
            out.push(ParsedLine {
                indent,
                kind: LineKind::Scalar(body.to_string()),
            });
            continue;
        }
        if let Some(idx) = find_unquoted_colon(body) {
            let (k, v) = body.split_at(idx);
            let after = &v[1..];
            if after.trim().is_empty() {
                out.push(ParsedLine {
                    indent,
                    kind: LineKind::KeyOnly { key: k.to_string() },
                });
            } else {
                out.push(ParsedLine {
                    indent,
                    kind: LineKind::KeyValue {
                        key: k.to_string(),
                        value: after.trim_start().to_string(),
                    },
                });
            }
            continue;
        }
        out.push(ParsedLine {
            indent,
            kind: LineKind::Scalar(body.to_string()),
        });
    }
    out
}
