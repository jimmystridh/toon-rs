#![doc = include_str!("../README.md")]

pub mod error;
pub mod options;
pub mod encode;
pub mod decode;

#[cfg(feature = "serde")]
pub mod ser;
#[cfg(feature = "serde")]
pub mod de;

pub use crate::error::{Error, Result};
pub use crate::options::{Delimiter, Options};

use std::io::{Read, Write};

#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Serialize};

#[cfg(feature = "serde")]
pub fn encode_to_string<T: Serialize>(value: &T, options: &Options) -> Result<String> {
    crate::ser::to_string_streaming(value, options)
}

#[cfg(feature = "serde")]
pub fn encode_to_writer<W: Write, T: Serialize>(mut writer: W, value: &T, options: &Options) -> Result<()> {
    let s = encode_to_string(value, options)?;
    writer.write_all(s.as_bytes())?;
    Ok(())
}

#[cfg(feature = "serde")]
pub fn decode_from_str<T: DeserializeOwned>(s: &str, options: &Options) -> Result<T> {
    if options.strict {
        let lines = crate::decode::scanner::scan(s);
        if let Err(e) = crate::decode::validation::validate_indentation(&lines) {
            return Err(crate::error::Error::Syntax { line: e.line, message: e.message });
        }
    }
    let v = crate::decode::parser::parse_to_value_with_strict(s, options.strict)?;
    let t = serde_json::from_value(v)?;
    Ok(t)
}

#[cfg(feature = "serde")]
pub fn decode_from_reader<R: Read, T: DeserializeOwned>(mut reader: R, options: &Options) -> Result<T> {
    let mut s = String::new();
    reader.read_to_string(&mut s)?;
    decode_from_str(&s, options)
}
