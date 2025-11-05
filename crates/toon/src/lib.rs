#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod encode;
pub mod error;
pub mod options;
pub mod value;

pub mod decode;

#[cfg(feature = "serde")]
pub mod de;
#[cfg(feature = "serde")]
pub mod ser;

pub use crate::error::{Error, Result};
pub use crate::options::{Delimiter, Options};

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "std")]
use std::io::{Read, Write};

#[cfg(feature = "serde")]
use serde::{Serialize, de::DeserializeOwned};

#[cfg(feature = "serde")]
pub fn encode_to_string<T: Serialize>(value: &T, options: &Options) -> Result<String> {
    crate::ser::to_string_streaming(value, options)
}

#[cfg(all(feature = "serde", feature = "std"))]
pub fn encode_to_writer<W: Write, T: Serialize>(
    mut writer: W,
    value: &T,
    options: &Options,
) -> Result<()> {
    let s = encode_to_string(value, options)?;
    writer.write_all(s.as_bytes())?;
    Ok(())
}

// Decoding helpers require the json (serde_json) feature
#[cfg(feature = "serde")]
pub fn decode_from_str<T: DeserializeOwned + 'static>(s: &str, options: &Options) -> Result<T> {
    crate::de::from_str(s, options)
}

#[cfg(all(feature = "serde", feature = "std"))]
pub fn decode_from_reader<R: Read, T: DeserializeOwned + 'static>(
    mut reader: R,
    options: &Options,
) -> Result<T> {
    let mut s = String::new();
    reader.read_to_string(&mut s)?;
    decode_from_str(&s, options)
}
