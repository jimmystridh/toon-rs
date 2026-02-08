//! Strict-mode validation
#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

#[derive(Debug)]
pub struct ValidationError {
    pub line: usize,
    pub message: String,
}

/// Validate indentation with configurable indent size (typically 2 or 4)
pub fn validate_indentation_with_size<'a>(
    lines: &[crate::decode::scanner::ParsedLine<'a>],
    raw_lines: &[&str],
    indent_size: usize,
) -> Result<(), ValidationError> {
    let indent_size = if indent_size == 0 { 2 } else { indent_size };

    for (idx, pl) in lines.iter().enumerate() {
        use crate::decode::scanner::LineKind;
        if matches!(pl.kind, LineKind::Blank) {
            continue;
        }

        // Check for tabs in indentation (strict mode forbids tabs)
        if idx < raw_lines.len() {
            let raw = raw_lines[idx];
            for c in raw.chars() {
                if c != ' ' && c != '\t' {
                    break;
                }
                if c == '\t' {
                    return Err(ValidationError {
                        line: idx + 1,
                        message: "tab character used in indentation".to_string(),
                    });
                }
            }
        }

        // §14.3: Leading spaces must be an exact multiple of indentSize
        if pl.indent % indent_size != 0 {
            return Err(ValidationError {
                line: idx + 1,
                message: format!(
                    "indentation ({}) must be a multiple of {}",
                    pl.indent, indent_size
                ),
            });
        }
    }
    Ok(())
}

/// Legacy function for backwards compatibility (uses indent=2)
pub fn validate_indentation<'a>(
    lines: &[crate::decode::scanner::ParsedLine<'a>],
) -> Result<(), ValidationError> {
    // Without raw lines, we can't check tabs. Use new function with raw lines for full validation.
    for (idx, pl) in lines.iter().enumerate() {
        use crate::decode::scanner::LineKind;
        if matches!(pl.kind, LineKind::Blank) {
            continue;
        }
        // §14.3: Leading spaces must be an exact multiple of indentSize (2)
        if pl.indent % 2 != 0 {
            return Err(ValidationError {
                line: idx + 1,
                message: format!("indentation ({}) must be a multiple of 2", pl.indent),
            });
        }
    }
    Ok(())
}
