#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Delimiter {
    #[default]
    Comma,
    Tab,
    Pipe,
}

#[derive(Debug, Clone, Default)]
pub struct Options {
    pub delimiter: Delimiter,
    pub strict: bool,
}
