#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

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

/// Returns the delimiter symbol for use inside bracket segments.
/// Comma is represented by absence (returns None), tab and pipe are explicit.
pub fn delimiter_symbol(delim: Delimiter) -> Option<char> {
    match delim {
        Delimiter::Comma => None,
        Delimiter::Tab => Some('\t'),
        Delimiter::Pipe => Some('|'),
    }
}

/// Format an array header bracket segment: `[N]` or `[N<delim>]`
pub fn format_bracket_segment(len: usize, delim: Delimiter) -> String {
    match delimiter_symbol(delim) {
        Some(sym) => format!("[{}{}]", len, sym),
        None => format!("[{}]", len),
    }
}

/// Format a tabular fields segment: `{f1,f2}` or `{f1<delim>f2}`
pub fn format_fields_segment(fields: &[String], delim: Delimiter) -> String {
    let dch = delimiter_char(delim);
    let mut out = String::from("{");
    for (i, f) in fields.iter().enumerate() {
        if i > 0 {
            out.push(dch);
        }
        out.push_str(f);
    }
    out.push('}');
    out
}

/// Format a complete array header for inline primitive arrays: `[N]: ` or `[N<delim>]: `
pub fn format_inline_array_header(len: usize, delim: Delimiter) -> String {
    format!("{}: ", format_bracket_segment(len, delim))
}

/// Format a complete tabular array header: `[N]{f1,f2}:` or `[N<delim>]{f1<delim>f2}:`
pub fn format_tabular_header(len: usize, fields: &[String], delim: Delimiter) -> String {
    format!(
        "{}{}:",
        format_bracket_segment(len, delim),
        format_fields_segment(fields, delim)
    )
}

/// Format an expanded array header (no inline values): `[N]:` or `[N<delim>]:`
pub fn format_expanded_array_header(len: usize, delim: Delimiter) -> String {
    format!("{}:", format_bracket_segment(len, delim))
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
    // Any string starting with hyphen must be quoted (§7.2)
    if s.starts_with('-') {
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
    // Brackets and braces are structural characters (§7.2)
    if s.chars().any(|c| matches!(c, '[' | ']' | '{' | '}')) {
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

/// Check if a key needs quoting per §7.3
/// Keys MAY be unquoted only if they match: ^[A-Za-z_][A-Za-z0-9_.]*$
fn key_needs_quotes(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    let mut chars = s.chars();
    // First character must be letter or underscore
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return true,
    }
    // Rest must be alphanumeric, underscore, or dot
    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' && c != '.' {
            return true;
        }
    }
    false
}

/// Format a key for output (§7.3)
pub fn format_key(s: &str) -> String {
    if key_needs_quotes(s) {
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
