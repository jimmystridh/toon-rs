//! Encoding pipeline for TOON (phase 1: primitives, objects, arrays; no tabular emission yet)

pub mod primitives;
pub mod writer;
pub mod encoders;
pub mod normalize;

use crate::{options::Options, Result};

#[cfg(feature = "serde")]
use serde_json::Value;

#[cfg(feature = "serde")]
pub fn encode_value_to_string(value: &Value, options: &Options) -> Result<String> {
    let mut w = writer::LineWriter::new();
    let v = crate::encode::normalize::normalize_value(value);
    encoders::encode_value(&v, &mut w, options, 0)?;
    Ok(w.into_string())
}
