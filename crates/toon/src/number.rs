#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

/// Format a finite f64 in canonical TOON form.
/// Requirements:
/// - no exponent notation
/// - no trailing fractional zeros (strip decimal point if none remains)
/// - no leading zeros except a single zero before the decimal point
/// - -0 normalized to 0
pub(crate) fn format_canonical_f64(value: f64) -> String {
    if !value.is_finite() {
        debug_assert!(false, "format_canonical_f64 called with non-finite value");
        return String::from("null");
    }
    if value == 0.0 {
        return String::from("0");
    }

    let mut sign_prefix = "";
    let mut magnitude = value;
    if magnitude < 0.0 {
        sign_prefix = "-";
        magnitude = -magnitude;
    }

    let mut buf = ryu::Buffer::new();
    let raw = buf.format_finite(magnitude);
    let body = if let Some(exp_index) = raw.find(['e', 'E']) {
        let mantissa = &raw[..exp_index];
        let exp: i32 = raw[exp_index + 1..].parse().unwrap_or(0);
        expand_exponent(mantissa, exp)
    } else {
        String::from(raw)
    };
    let trimmed = trim_fraction(body);
    if trimmed == "0" {
        // Handle cases where normalization produced 0 (e.g., -0.0)
        return String::from("0");
    }
    if sign_prefix.is_empty() {
        trimmed
    } else {
        let mut out = String::with_capacity(sign_prefix.len() + trimmed.len());
        out.push('-');
        out.push_str(&trimmed);
        out
    }
}

pub(crate) fn has_forbidden_leading_zeros(token: &str) -> bool {
    let token = token.trim();
    if token.is_empty() {
        return false;
    }
    let token = token.strip_prefix('-').unwrap_or(token);
    let token = token.strip_prefix('+').unwrap_or(token);

    // Allow single zero and zero immediately followed by '.' or exponent.
    if token.len() <= 1 {
        return false;
    }
    let first = token.as_bytes()[0];
    if first != b'0' {
        return false;
    }
    let second = token.as_bytes()[1];
    if second == b'.' || second == b'e' || second == b'E' {
        return false;
    }
    true
}

fn expand_exponent(mantissa: &str, exp: i32) -> String {
    let mut digits = Vec::with_capacity(mantissa.len());
    let mut point_index = mantissa.len();
    for &b in mantissa.as_bytes() {
        if b == b'.' {
            point_index = digits.len();
        } else {
            digits.push(b);
        }
    }
    if point_index == mantissa.len() {
        point_index = digits.len();
    }

    if exp >= 0 {
        let target = point_index as i32 + exp;
        if target >= digits.len() as i32 {
            let mut result = String::with_capacity(target as usize);
            for &d in &digits {
                result.push(d as char);
            }
            let zeros = (target as usize).saturating_sub(digits.len());
            for _ in 0..zeros {
                result.push('0');
            }
            result
        } else {
            let split = target as usize;
            let mut result = String::with_capacity(digits.len() + 1);
            for (idx, &d) in digits.iter().enumerate() {
                if idx == split {
                    result.push('.');
                }
                result.push(d as char);
            }
            result
        }
    } else {
        let shift = (-exp) as usize;
        if shift >= point_index {
            let zeros = shift - point_index;
            let mut result = String::with_capacity(digits.len() + zeros + 2);
            result.push_str("0.");
            for _ in 0..zeros {
                result.push('0');
            }
            for &d in &digits {
                result.push(d as char);
            }
            result
        } else {
            let split = point_index - shift;
            let mut result = String::with_capacity(digits.len() + 1);
            for (idx, &d) in digits.iter().enumerate() {
                if idx == split {
                    result.push('.');
                }
                result.push(d as char);
            }
            result
        }
    }
}

fn trim_fraction(mut s: String) -> String {
    if let Some(dot_pos) = s.find('.') {
        let mut end = s.len();
        while end > dot_pos + 1 && s.as_bytes()[end - 1] == b'0' {
            end -= 1;
        }
        if end > dot_pos && s.as_bytes()[end - 1] == b'.' {
            end -= 1;
        }
        s.truncate(end);
    }
    s
}
