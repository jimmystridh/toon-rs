//! Strict-mode validation (basic)
#[derive(Debug, Clone, Copy, Default)]
pub struct StrictConfig {
    pub enabled: bool,
}

#[derive(Debug)]
pub struct ValidationError {
    pub line: usize,
    pub message: String,
}

pub fn validate_indentation(lines: &[crate::decode::scanner::ParsedLine]) -> Result<(), ValidationError> {
    let mut prev_indent: Option<usize> = None;
    for (idx, pl) in lines.iter().enumerate() {
        use crate::decode::scanner::LineKind;
        if matches!(pl.kind, LineKind::Blank) { continue; }
        if let Some(pi) = prev_indent {
            if pl.indent > pi {
                if pl.indent != pi + 2 {
                    return Err(ValidationError { line: idx + 1, message: format!("indent increase must be +2, got {}->{}", pi, pl.indent) });
                }
            } else if pl.indent < pi {
                if (pi - pl.indent) % 2 != 0 {
                    return Err(ValidationError { line: idx + 1, message: format!("indent decrease must be multiple of 2, got {}->{}", pi, pl.indent) });
                }
            }
        }
        prev_indent = Some(pl.indent);
    }
    Ok(())
}
