#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

#[cfg(feature = "std")]
use std::string::String;

use crate::number::format_canonical_f64;
use crate::options::Delimiter;

pub fn delimiter_char(delim: Delimiter) -> char {
    match delim {
        Delimiter::Comma => ',',
        Delimiter::Tab => '\t',
        Delimiter::Pipe => '|',
    }
}

fn is_control(c: char) -> bool {
    let u = c as u32;
    u < 0x20 || u == 0x7F
}

fn looks_like_literal(s: &str) -> bool {
    if matches!(s, "true" | "false" | "null") {
        return true;
    }
    let sn = s.trim();
    if sn.starts_with('+') || sn.starts_with('-') {
        return sn[1..].parse::<f64>().is_ok();
    }
    sn.parse::<f64>().is_ok()
}

pub fn needs_quotes(s: &str, delim: Delimiter) -> bool {
    if s.is_empty() {
        return true;
    }
    // A lone hyphen is a list item marker and must be quoted
    if s == "-" {
        return true;
    }
    if s.starts_with('-') && s.len() >= 2 && s.as_bytes()[1] == b' ' {
        return true;
    }
    // Keys starting with @ could be confused with tabular array headers
    if s.starts_with('@') {
        return true;
    }
    if s.starts_with(' ') || s.ends_with(' ') {
        return true;
    }
    if s.contains(delimiter_char(delim)) {
        return true;
    }
    if s.contains(':') {
        return true;
    }
    if s.chars().any(|c| c == '"' || c == '\\' || is_control(c)) {
        return true;
    }
    if looks_like_literal(s) {
        return true;
    }
    false
}

pub fn escape_and_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => {
                out.push('\\');
                out.push('"');
            }
            '\\' => {
                out.push('\\');
                out.push('\\');
            }
            '\n' => {
                out.push_str("\\n");
            }
            '\r' => {
                out.push_str("\\r");
            }
            '\t' => {
                out.push_str("\\t");
            }
            c if is_control(c) => {
                use core::fmt::Write as _;
                let _ = write!(out, "\\u{:04X}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

pub fn format_string(s: &str, delim: Delimiter) -> String {
    if needs_quotes(s, delim) {
        escape_and_quote(s)
    } else {
        s.to_string()
    }
}

pub fn format_bool(b: bool) -> &'static str {
    if b { "true" } else { "false" }
}

pub fn format_null() -> &'static str {
    "null"
}

pub fn format_f64(f: f64) -> String {
    format_canonical_f64(f)
}
