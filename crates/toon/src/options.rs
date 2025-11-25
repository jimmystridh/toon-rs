#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Delimiter {
    #[default]
    Comma,
    Tab,
    Pipe,
}

/// Key folding mode for encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyFolding {
    /// No key folding - standard nested encoding
    #[default]
    Off,
    /// Safe key folding - collapse single-key chains, skip if segment needs quotes
    Safe,
}

/// Path expansion mode for decoding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExpandPaths {
    /// No path expansion - dotted keys are preserved as-is
    #[default]
    Off,
    /// Safe path expansion - expand dotted keys to nested objects
    /// Only expands keys where all segments are valid identifiers (no quotes needed)
    Safe,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub delimiter: Delimiter,
    pub strict: bool,
    /// Indentation size (default: 2 spaces)
    pub indent: usize,
    /// Key folding mode for encoding
    pub key_folding: KeyFolding,
    /// Maximum depth for key folding (None = unlimited, Some(0) = disabled)
    pub flatten_depth: Option<usize>,
    /// Path expansion mode for decoding
    pub expand_paths: ExpandPaths,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            delimiter: Delimiter::default(),
            strict: false,
            indent: 2,
            key_folding: KeyFolding::Off,
            flatten_depth: None,
            expand_paths: ExpandPaths::Off,
        }
    }
}
