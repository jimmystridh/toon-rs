#[cfg(not(feature = "std"))]
use alloc::string::String;

use crate::options::Delimiter;

pub struct LineWriter {
    out: String,
    indent_cache: String,
    scratch: String,
}

impl LineWriter {
    pub fn new() -> Self {
        Self {
            out: String::new(),
            indent_cache: String::new(),
            scratch: String::new(),
        }
    }

    fn write_indent(&mut self, indent: usize) {
        if indent == 0 {
            return;
        }
        if self.indent_cache.len() < indent {
            self.indent_cache
                .extend(core::iter::repeat(' ').take(indent - self.indent_cache.len()));
        }
        self.out.push_str(&self.indent_cache[..indent]);
    }

    fn write_formatted(&mut self, value: &str, delim: Delimiter) {
        if crate::encode::primitives::needs_quotes(value, delim) {
            self.scratch.clear();
            crate::encode::primitives::escape_and_quote_into(&mut self.scratch, value);
            self.out.push_str(&self.scratch);
        } else {
            self.out.push_str(value);
        }
    }

    pub fn line_formatted(&mut self, indent: usize, s: &str, delim: Delimiter) {
        self.write_indent(indent);
        self.write_formatted(s, delim);
        self.out.push('\n');
    }

    pub fn line_kv_formatted(&mut self, indent: usize, key: &str, value: &str, delim: Delimiter) {
        self.write_indent(indent);
        self.write_formatted(key, delim);
        self.out.push_str(": ");
        self.write_formatted(value, delim);
        self.out.push('\n');
    }

    pub fn line_kv_key_formatted_raw(
        &mut self,
        indent: usize,
        key: &str,
        value: &str,
        delim: Delimiter,
    ) {
        self.write_indent(indent);
        self.write_formatted(key, delim);
        self.out.push_str(": ");
        self.out.push_str(value);
        self.out.push('\n');
    }

    pub fn line_list_item_formatted(&mut self, indent: usize, value: &str, delim: Delimiter) {
        self.write_indent(indent);
        self.out.push_str("- ");
        self.write_formatted(value, delim);
        self.out.push('\n');
    }

    pub fn line_key_only_formatted(&mut self, indent: usize, key: &str, delim: Delimiter) {
        self.write_indent(indent);
        self.write_formatted(key, delim);
        self.out.push(':');
        self.out.push('\n');
    }

    pub fn line(&mut self, indent: usize, s: &str) {
        self.write_indent(indent);
        self.out.push_str(s);
        self.out.push('\n');
    }

    pub fn line_kv(&mut self, indent: usize, key: &str, value: &str) {
        self.write_indent(indent);
        self.out.push_str(key);
        self.out.push_str(": ");
        self.out.push_str(value);
        self.out.push('\n');
    }

    pub fn line_list_item(&mut self, indent: usize, value: &str) {
        self.write_indent(indent);
        self.out.push_str("- ");
        self.out.push_str(value);
        self.out.push('\n');
    }

    pub fn line_key_only(&mut self, indent: usize, key: &str) {
        self.write_indent(indent);
        self.out.push_str(key);
        self.out.push(':');
        self.out.push('\n');
    }

    pub fn into_string(self) -> String {
        self.out
    }
}

impl Default for LineWriter {
    fn default() -> Self {
        Self::new()
    }
}
