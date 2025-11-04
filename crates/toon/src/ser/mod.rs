//! Serde encoding helpers for TOON

use crate::{encode::encode_value_to_string, options::Options, Result};

#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg(feature = "serde")]
mod value_builder;
mod stream;

#[cfg(feature = "serde")]
pub fn to_string<T: Serialize>(value: &T, options: &Options) -> Result<String> {
    let v = value_builder::to_value(value, options);
    encode_value_to_string(&v, options)
}

#[cfg(feature = "serde")]
pub fn to_writer<W: std::io::Write, T: Serialize>(mut writer: W, value: &T, options: &Options) -> Result<()> {
    let s = to_string(value, options)?;
    std::io::Write::write_all(&mut writer, s.as_bytes())?;
    Ok(())
}

pub fn to_string_streaming<T: Serialize>(value: &T, options: &Options) -> Result<String> {
    stream::to_string_streaming(value, options)
}

pub fn to_writer_streaming<W: std::io::Write, T: Serialize>(mut writer: W, value: &T, options: &Options) -> Result<()> {
    let s = to_string_streaming(value, options)?;
    std::io::Write::write_all(&mut writer, s.as_bytes())?;
    Ok(())
}
