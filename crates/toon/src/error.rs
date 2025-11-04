#[cfg(feature = "std")]
use thiserror::Error;

#[cfg(feature = "std")]
use std::io;

#[cfg(feature = "std")]
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[cfg(all(feature = "serde", feature = "json"))]
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("syntax at line {line}: {message}")]
    Syntax { line: usize, message: String },

    #[error("{0}")]
    Message(String),
}

#[cfg(not(feature = "std"))]
#[derive(Debug)]
pub enum Error {
    Syntax { line: usize, message: alloc::string::String },
    Message(alloc::string::String),
}

#[cfg(not(feature = "std"))]
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Syntax { line, message } => write!(f, "syntax at line {}: {}", line, message),
            Error::Message(m) => f.write_str(m),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
