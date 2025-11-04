#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    Comma,
    Tab,
    Pipe,
}

impl Default for Delimiter {
    fn default() -> Self {
        Delimiter::Comma
    }
}

#[derive(Debug, Clone)]
pub struct Options {
    pub delimiter: Delimiter,
    pub strict: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self { delimiter: Delimiter::Comma, strict: false }
    }
}
