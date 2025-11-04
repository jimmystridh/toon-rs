//! Encoding pipeline for TOON (phase 1: primitives, objects, arrays; no tabular emission yet)

pub mod primitives;
pub mod writer;
#[cfg(feature = "json")]
pub mod encoders;
#[cfg(feature = "json")]
pub mod normalize;

use crate::{options::Options, Result};

#[cfg(all(feature = "serde", feature = "json"))]
use serde_json::Value;

#[cfg(all(feature = "serde", feature = "json"))]
pub fn encode_value_to_string(value: &Value, options: &Options) -> Result<String> {
    let mut w = writer::LineWriter::new();
    let v = crate::encode::normalize::normalize_value(value);
    encoders::encode_value(&v, &mut w, options, 0)?;
    Ok(w.into_string())
}
