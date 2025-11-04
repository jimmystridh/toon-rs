//! Serde encoding helpers for TOON

#[cfg(not(feature = "std"))]
use alloc::string::String;

use crate::{options::Options, Result};

#[cfg(feature = "serde")]
use serde::Serialize;

// serde_json Value-based builder is only available with feature "json"
#[cfg(all(feature = "serde", feature = "json"))]
mod value_builder;
#[cfg(all(feature = "serde", not(feature = "json")))]
mod value_builder_alloc;
mod stream;

// Non-streaming encoding via serde_json::Value is only available with "json"
#[cfg(all(feature = "serde", feature = "json"))]
pub fn to_string<T: Serialize>(value: &T, options: &Options) -> Result<String> {
    let v = value_builder::to_value(value, options);
    crate::encode::encode_value_to_string(&v, options)
}

// Writer variant requires std
#[cfg(all(feature = "serde", feature = "json", feature = "std"))]
pub fn to_writer<W: std::io::Write, T: Serialize>(mut writer: W, value: &T, options: &Options) -> Result<()> {
    let s = to_string(value, options)?;
    std::io::Write::write_all(&mut writer, s.as_bytes())?;
    Ok(())
}

#[cfg(feature = "serde")]
pub fn to_string_streaming<T: Serialize>(value: &T, options: &Options) -> Result<String> {
    stream::to_string_streaming(value, options)
}

#[cfg(all(feature = "serde", feature = "std"))]
pub fn to_writer_streaming<W: std::io::Write, T: Serialize>(mut writer: W, value: &T, options: &Options) -> Result<()> {
    let s = to_string_streaming(value, options)?;
    std::io::Write::write_all(&mut writer, s.as_bytes())?;
    Ok(())
}
