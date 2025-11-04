pub struct LineWriter {
    out: String,
}

impl LineWriter {
    pub fn new() -> Self { Self { out: String::new() } }

    fn write_indent(&mut self, indent: usize) {
        for _ in 0..indent { self.out.push(' '); }
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
        self.out.push_str(":");
        self.out.push('\n');
    }

    pub fn into_string(self) -> String { self.out }
}