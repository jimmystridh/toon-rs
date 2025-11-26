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
    let mut prev_indent: Option<usize> = None;

    for (idx, pl) in lines.iter().enumerate() {
        use crate::decode::scanner::LineKind;
        if matches!(pl.kind, LineKind::Blank) {
            continue;
        }

        // Check for tabs in indentation (strict mode forbids tabs)
        if idx < raw_lines.len() {
            let raw = raw_lines[idx];
            // Check if there are any tabs in the leading whitespace
            // Note: pl.indent is count of spaces, so if raw has tabs before pl.indent chars,
            // it means tabs were used
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

        if let Some(pi) = prev_indent {
            if pl.indent > pi {
                if pl.indent != pi + indent_size {
                    return Err(ValidationError {
                        line: idx + 1,
                        message: format!(
                            "indent increase must be +{}, got {}->{}",
                            indent_size, pi, pl.indent
                        ),
                    });
                }
            } else if pl.indent < pi && (pi - pl.indent) % indent_size != 0 {
                return Err(ValidationError {
                    line: idx + 1,
                    message: format!(
                        "indent decrease must be multiple of {}, got {}->{}",
                        indent_size, pi, pl.indent
                    ),
                });
            }
        }
        prev_indent = Some(pl.indent);
    }
    Ok(())
}

/// Legacy function for backwards compatibility (uses indent=2)
pub fn validate_indentation<'a>(
    lines: &[crate::decode::scanner::ParsedLine<'a>],
) -> Result<(), ValidationError> {
    // Without raw lines, we can't check tabs. Use new function with raw lines for full validation.
    let mut prev_indent: Option<usize> = None;
    for (idx, pl) in lines.iter().enumerate() {
        use crate::decode::scanner::LineKind;
        if matches!(pl.kind, LineKind::Blank) {
            continue;
        }
        if let Some(pi) = prev_indent {
            if pl.indent > pi {
                if pl.indent != pi + 2 {
                    return Err(ValidationError {
                        line: idx + 1,
                        message: format!("indent increase must be +2, got {}->{}", pi, pl.indent),
                    });
                }
            } else if pl.indent < pi && (pi - pl.indent) % 2 != 0 {
                return Err(ValidationError {
                    line: idx + 1,
                    message: format!(
                        "indent decrease must be multiple of 2, got {}->{}",
                        pi, pl.indent
                    ),
                });
            }
        }
        prev_indent = Some(pl.indent);
    }
    Ok(())
}
